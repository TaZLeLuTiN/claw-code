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
        let requested = args.get("model").and_then(|v| v.as_str()).unwrap_or("auto");
        if requested == "auto" {
            let classification = classify(&prompt, &context_files);
            tracing::info!(
                provider = ?classification.provider,
                model = %classification.model,
                confidence = classification.confidence,
                "Classification result"
            );
            classification.model
        } else {
            requested.to_string()
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

    let result = classify(prompt, &context_files);

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

async fn call_ollama(host: &str, model: &str, prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", host.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    });

    let resp = client
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
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
