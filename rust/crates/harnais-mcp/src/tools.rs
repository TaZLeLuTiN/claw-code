//! MCP tools: ollama_generate and ollama_route.

use anyhow::Result;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use crate::classifier::classify;
use crate::server::JsonRpcResponse;

pub async fn ollama_generate(
    id: Option<Value>,
    args: Value,
    ollama_host: &str,
    pg_dsn: &Option<String>,
) -> JsonRpcResponse {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return JsonRpcResponse::err(id, -32602, "Missing 'prompt' argument".to_string()),
    };

    let context_files: Vec<String> = args
        .get("context_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let model = {
        let requested_model = args.get("model").and_then(|v| v.as_str()).unwrap_or("auto");
        let task_type_arg = args
            .get("task_type")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        if requested_model != "auto" {
            requested_model.to_string()
        } else if task_type_arg != "auto" {
            // Map task_type string → TaskType, then use select_model for file-aware routing
            let task_type = match task_type_arg {
                "implementation" => crate::classifier::TaskType::Implementation,
                "boilerplate" => crate::classifier::TaskType::Boilerplate,
                "tests" => crate::classifier::TaskType::Tests,
                "distillation" => crate::classifier::TaskType::Distillation,
                "code_algo" => crate::classifier::TaskType::CodeAlgo,
                _ => crate::classifier::TaskType::Boilerplate,
            };
            crate::classifier::select_model(&task_type, &context_files)
        } else {
            let classification = classify(&prompt, &context_files);
            tracing::info!(
                provider = ?classification.provider,
                model = %classification.model,
                confidence = classification.confidence,
                "Classification result"
            );
            // If classifier selected Claude, ollama_generate must still call Ollama
            if classification.provider == crate::classifier::Provider::Claude {
                tracing::warn!(
                    "Classifier selected Claude for ollama_generate — using gemma4:31b as fallback"
                );
                "gemma4:31b".to_string()
            } else {
                crate::classifier::select_model(&classification.task_type, &context_files)
            }
        }
    };

    let start = Instant::now();
    let result = call_ollama(ollama_host, &model, &prompt).await;
    let duration_ms = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);

    match result {
        Ok(response_text) => {
            if let Some(dsn) = pg_dsn {
                let _ =
                    log_routing_decision(dsn, &prompt, "ollama", &model, duration_ms, "success")
                        .await;
            }
            JsonRpcResponse::ok(
                id,
                serde_json::json!({
                    "content": [{ "type": "text", "text": response_text }],
                    "isError": false,
                    "_meta": {
                        "model": model,
                        "duration_ms": duration_ms,
                        "provider": "ollama"
                    }
                }),
            )
        }
        Err(e) => {
            tracing::error!("Ollama call failed: {e}");
            if let Some(dsn) = pg_dsn {
                let _ = log_routing_decision(dsn, &prompt, "ollama", &model, duration_ms, "failed")
                    .await;
            }
            JsonRpcResponse::err(id, -32603, format!("Ollama error: {e}"))
        }
    }
}

#[must_use]
pub fn ollama_route(id: Option<Value>, args: Value) -> JsonRpcResponse {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return JsonRpcResponse::err(id, -32602, "Missing 'prompt'".to_string()),
    };

    let context_files: Vec<String> = args
        .get("context_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut result = classify(prompt, &context_files);
    if result.provider == crate::classifier::Provider::Ollama {
        result.model = crate::classifier::select_model(&result.task_type, &context_files);
    }

    JsonRpcResponse::ok(
        id,
        serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result).unwrap_or_default()
            }]
        }),
    )
}

/// Compute a dynamic HTTP timeout based on model speed and prompt length.
///
/// Formula: (input_tokens + expected_output_tokens) / tokens_per_second * safety_factor
///
/// Estimated tokens/sec on Apple M-series (empirical, conservative):
///   gemma3:4b=60, gemma3:12b=25, gemma4:26b=15, gemma4:31b=12,
///   qwen2.5:14b=18, qwen2.5:32b=8, 70b=4, default=15
pub fn compute_timeout(model: &str, prompt: &str) -> std::time::Duration {
    let tokens_per_sec: f64 = if model.contains("3:4b") {
        60.0
    } else if model.contains("3:12b") {
        25.0
    } else if model.contains("4:26b") {
        15.0
    } else if model.contains("4:31b") {
        12.0
    } else if model.contains("qwen2.5:14") {
        18.0
    } else if model.contains("qwen2.5:32") {
        8.0
    } else if model.contains("70b") {
        4.0
    } else {
        15.0
    };

    let input_tokens = (prompt.len() / 4) as f64;
    let expected_output_tokens: f64 = 600.0;
    let safety_factor: f64 = 2.5;

    let raw_secs =
        ((input_tokens + expected_output_tokens) / tokens_per_sec * safety_factor) as u64;

    std::time::Duration::from_secs(raw_secs.clamp(30, 600))
}

async fn call_ollama(host: &str, model: &str, prompt: &str) -> Result<String> {
    match call_ollama_raw(host, model, prompt).await {
        Ok(text) => Ok(text),
        Err(e) if is_connect_error(&e) && host.contains("localhost") => {
            let fallback = host.replace("localhost", "127.0.0.1");
            tracing::warn!("Ollama unreachable via localhost (IPv6?), retrying on {fallback}");
            call_ollama_raw(&fallback, model, prompt).await
        }
        Err(e) => Err(e),
    }
}

fn is_connect_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<reqwest::Error>()
        .map(|re| re.is_connect())
        .unwrap_or(false)
}

async fn call_ollama_raw(host: &str, model: &str, prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", host.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    });

    let timeout = compute_timeout(model, prompt);
    tracing::debug!(
        model,
        prompt_len = prompt.len(),
        timeout_secs = timeout.as_secs(),
        "Dynamic timeout computed"
    );

    let resp = client
        .post(&url)
        .json(&body)
        .timeout(timeout)
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Ollama HTTP {}: {}", resp.status(), resp.text().await?);
    }

    let json: serde_json::Value = resp.json().await?;
    let response_text = json["response"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'response' field in Ollama output"))?
        .to_string();

    Ok(response_text)
}

async fn log_routing_decision(
    pg_dsn: &str,
    prompt: &str,
    provider: &str,
    model: &str,
    duration_ms: i64,
    result: &str,
) -> Result<()> {
    let mut h = DefaultHasher::new();
    prompt.hash(&mut h);
    let prompt_hash = format!("{:x}", h.finish());
    let prompt_len = i32::try_from(prompt.len()).unwrap_or(i32::MAX);
    let duration_pg = i32::try_from(duration_ms).unwrap_or(i32::MAX);

    let (client, connection) = tokio_postgres::connect(pg_dsn, tokio_postgres::NoTls).await?;
    drop(tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("PG connection error: {e}");
        }
    }));

    client
        .execute(
            "INSERT INTO gates.routing_decisions
             (project, prompt_hash, prompt_len, task_type, provider, model, duration_ms, result)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &"harnais-mcp",
                &prompt_hash,
                &prompt_len,
                &"auto",
                &provider,
                &model,
                &duration_pg,
                &result,
            ],
        )
        .await?;

    Ok(())
}
