use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("HARNAIS_MCP_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let ollama_host =
        env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let pg_dsn = env::var("PG_HARNAIS_DSN").ok();

    harnais_mcp::server::run_server(ollama_host, pg_dsn).await
}
