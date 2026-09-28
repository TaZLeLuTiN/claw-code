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
            JsonRpcResponse::err(id, -32603, format!("Ollama error: {e:#}"))
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

/// Seuils de SILENCE d'une génération Ollama. Une génération qui PROGRESSE n'est jamais coupée, quelle que soit sa
/// durée ; seule une génération MUETTE l'est.
///
/// Remplace `compute_timeout` (retiré le 2026-09-28) : il PRÉDISAIT la durée avec des vitesses écrites en dur
/// (`gemma4:31b` = 12 tok/s) et 600 jetons de sortie. Mesuré sur le Mac : 6,3 tok/s et 1942 jetons pour un prompt
/// de 4,6 Ko (341 s) — `cc-symphony` a vu gemma4 échouer 2 fois sur 2 au-delà du délai prédit.
#[derive(Debug, Clone, Copy)]
pub struct Silences {
    /// Avant le PREMIER jeton : chargement du modèle + lecture du prompt (mesuré : 15 s + 20 s pour 1,5 k jetons).
    pub premier: std::time::Duration,
    /// Entre deux morceaux, une fois la génération partie.
    pub entre: std::time::Duration,
}

impl Default for Silences {
    fn default() -> Self {
        Self {
            premier: std::time::Duration::from_secs(600),
            entre: std::time::Duration::from_secs(120),
        }
    }
}

/// Assemble le flux NDJSON d'`/api/generate` (`stream: true`), coupé n'importe où par le transport.
#[derive(Debug, Default)]
pub struct Flux {
    tampon: String,
    texte: String,
    fini: bool,
    eval_count: Option<u64>,
    eval_duration_ns: Option<u64>,
}

impl Flux {
    /// Ajoute des octets reçus ; chaque ligne COMPLÈTE est lue, le reste attend la suite.
    pub fn pousser(&mut self, octets: &[u8]) -> Result<()> {
        self.tampon.push_str(&String::from_utf8_lossy(octets));
        while let Some(fin) = self.tampon.find('\n') {
            let ligne: String = self.tampon.drain(..=fin).collect();
            let ligne = ligne.trim();
            if ligne.is_empty() {
                continue;
            }
            let v: Value = serde_json::from_str(ligne)
                .map_err(|e| anyhow::anyhow!("ligne du flux Ollama illisible ({e}) : {ligne}"))?;
            if let Some(err) = v.get("error").and_then(Value::as_str) {
                anyhow::bail!("Ollama : {err}");
            }
            if let Some(r) = v.get("response").and_then(Value::as_str) {
                self.texte.push_str(r);
            }
            if v.get("done").and_then(Value::as_bool) == Some(true) {
                self.fini = true;
                self.eval_count = v.get("eval_count").and_then(Value::as_u64);
                self.eval_duration_ns = v.get("eval_duration").and_then(Value::as_u64);
            }
        }
        Ok(())
    }

    pub fn fini(&self) -> bool {
        self.fini
    }

    pub fn texte(&self) -> &str {
        &self.texte
    }

    /// Vitesse de sortie MESURÉE par Ollama (dernière ligne) ; `None` tant qu'elle n'est pas donnée — jamais supposée.
    #[allow(clippy::cast_precision_loss)]
    pub fn vitesse_tok_s(&self) -> Option<f64> {
        match (self.eval_count, self.eval_duration_ns) {
            (Some(n), Some(d)) if d > 0 => Some(n as f64 / (d as f64 / 1e9)),
            _ => None,
        }
    }
}

/// Une génération aboutie, avec ce qu'Ollama a MESURÉ.
#[derive(Debug)]
pub struct Generation {
    pub texte: String,
    pub tok_s: Option<f64>,
}

/// Génère en FLUX : abandonne sur un silence (`Silences`), jamais sur une durée totale prédite.
pub async fn generer_en_flux(
    host: &str,
    model: &str,
    prompt: &str,
    silences: Silences,
) -> Result<Generation> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;
    let url = format!("{}/api/generate", host.trim_end_matches('/'));
    let body = serde_json::json!({ "model": model, "prompt": prompt, "stream": true });

    let mut resp = tokio::time::timeout(silences.premier, client.post(&url).json(&body).send())
        .await
        .map_err(|_| muet(model, silences.premier, 0))??;
    if !resp.status().is_success() {
        anyhow::bail!("Ollama HTTP {}: {}", resp.status(), resp.text().await?);
    }
    let mut flux = Flux::default();
    loop {
        let seuil = if flux.texte().is_empty() {
            silences.premier
        } else {
            silences.entre
        };
        match tokio::time::timeout(seuil, resp.chunk()).await {
            Err(_) => return Err(muet(model, seuil, flux.texte().len())),
            Ok(Err(e)) => {
                return Err(
                    anyhow::Error::new(e).context(format!("flux Ollama interrompu ({model})"))
                )
            }
            Ok(Ok(Some(octets))) => flux.pousser(&octets)?,
            Ok(Ok(None)) => break,
        }
        if flux.fini() {
            break;
        }
    }
    if !flux.fini() {
        anyhow::bail!(
            "flux Ollama interrompu avant `done` ({model}, {} caractères reçus) : une réponse partielle n'est pas une réponse",
            flux.texte().len()
        );
    }
    let tok_s = flux.vitesse_tok_s();
    tracing::info!(
        model,
        tok_s = tok_s.unwrap_or(-1.0),
        "génération Ollama — vitesse MESURÉE"
    );
    Ok(Generation {
        texte: flux.texte,
        tok_s,
    })
}

fn muet(model: &str, seuil: std::time::Duration, recus: usize) -> anyhow::Error {
    anyhow::anyhow!(
        "Ollama MUET depuis {} s ({model}, {recus} caractères reçus) — abandonné sur SILENCE, pas sur lenteur",
        seuil.as_secs()
    )
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
    Ok(generer_en_flux(host, model, prompt, Silences::default())
        .await?
        .texte)
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
