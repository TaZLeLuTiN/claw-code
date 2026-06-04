//! MCP Server stdio — JSON-RPC 2.0 over stdin/stdout.
//! Compatible with Claude Code's MCP client.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    #[must_use]
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    #[must_use]
    pub fn err(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

pub async fn run_server(ollama_host: String, pg_dsn: Option<String>) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = tokio::io::BufWriter::new(stdout);

    tracing::info!("harnais-mcp server ready, listening on stdio");

    while let Some(line) = reader.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let (response, is_notification) = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(req) => {
                let is_notif = req.id.is_none();
                let resp = handle_request(req, &ollama_host, &pg_dsn).await;
                (resp, is_notif)
            }
            Err(e) => (
                JsonRpcResponse::err(None, -32700, format!("Parse error: {e}")),
                false,
            ),
        };

        if !is_notification {
            let response_str = serde_json::to_string(&response)?;
            writer.write_all(response_str.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
    }

    Ok(())
}

async fn handle_request(
    req: JsonRpcRequest,
    ollama_host: &str,
    pg_dsn: &Option<String>,
) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "notifications/initialized" => JsonRpcResponse::ok(req.id, serde_json::json!({})),
        "tools/list" => handle_tools_list(req.id),
        "tools/call" => handle_tools_call(req.id, req.params, ollama_host, pg_dsn).await,
        other => JsonRpcResponse::err(req.id, -32601, format!("Method not found: {other}")),
    }
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        id,
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "harnais-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        id,
        serde_json::json!({
            "tools": [
                {
                    "name": "ollama_generate",
                    "description": "Generate code or text using a local Ollama model. Use for implementation, boilerplate, tests, and distillation tasks.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prompt": {
                                "type": "string",
                                "description": "The prompt to send to the model"
                            },
                            "model": {
                                "type": "string",
                                "description": "Model to use. Use 'auto' to let the classifier decide.",
                                "default": "auto"
                            },
                            "task_type": {
                                "type": "string",
                                "enum": ["implementation","boilerplate","tests",
                                         "distillation","code_algo","auto"],
                                "default": "auto"
                            },
                            "context_files": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "File paths providing context for classification"
                            }
                        },
                        "required": ["prompt"]
                    }
                },
                {
                    "name": "ollama_route",
                    "description": "Decide which provider and model to use for a task, without executing it.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string" },
                            "context_files": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["prompt"]
                    }
                }
            ]
        }),
    )
}

async fn handle_tools_call(
    id: Option<Value>,
    params: Option<Value>,
    ollama_host: &str,
    pg_dsn: &Option<String>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::err(id, -32602, "Missing params".to_string()),
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return JsonRpcResponse::err(id, -32602, "Missing tool name".to_string()),
    };

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    match name.as_str() {
        "ollama_generate" => crate::tools::ollama_generate(id, args, ollama_host, pg_dsn).await,
        "ollama_route" => crate::tools::ollama_route(id, args),
        other => JsonRpcResponse::err(id, -32601, format!("Unknown tool: {other}")),
    }
}
