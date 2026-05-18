use crate::harnais::cli::{ContextArgs, ContextCommand};
use crate::harnais::config;
use std::io::Write as IoWrite;

type BoxError = Box<dyn std::error::Error>;

fn ffi_err(e: impl std::fmt::Display) -> BoxError {
    format!("FFI error: {e}").into()
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: ContextArgs) -> Result<(), BoxError> {
    match args.command {
        ContextCommand::Init(a) => handle_init(a.project.as_deref()),
        ContextCommand::Start(a) => handle_start(a.project, a.label),
    }
}

fn handle_init(project: Option<&str>) -> Result<(), BoxError> {
    let cfg = config::load_config()?;
    let name = project.unwrap_or(&cfg.harnais.project_name);
    let root = config::find_config_root().ok_or("No .harnais.toml found")?;

    let ctx_dir = root.join(".harnais").join("context").join(name);
    std::fs::create_dir_all(&ctx_dir)?;

    let manifest = ctx_dir.join("context.json");
    if !manifest.exists() {
        let content = serde_json::json!({
            "project": name,
            "initialized_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            "sessions": []
        });
        let mut f = std::fs::File::create(&manifest)?;
        writeln!(f, "{}", serde_json::to_string_pretty(&content)?)?;
    }

    println!("Context window initialized: {name}");
    println!("  Dir: {}", ctx_dir.display());
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn handle_start(project: String, label: Option<String>) -> Result<(), BoxError> {
    let cfg = config::load_config()?;

    // Load the always_load context files and ingest them into the CB
    let always_load = cfg.context.always_load;
    if always_load.is_empty() {
        println!("No always_load files configured — context start is a no-op.");
        return Ok(());
    }

    harnais_ffi::runtime::init_python_runtime().map_err(ffi_err)?;

    let root = config::find_config_root().ok_or("No .harnais.toml found")?;
    let session_label = label.unwrap_or_else(|| "session".to_string());
    let mut ingested = 0usize;

    for file_path in &always_load {
        let full_path = root.join(file_path);
        if !full_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&full_path)?;
        let result = harnais_ffi::context_broker::ingest_internal(
            project.clone(),
            content,
            "handoff".to_string(),
        )
        .map_err(ffi_err)?;
        if result.action != "duplicate" {
            ingested += 1;
        }
    }

    println!("Context started for '{project}' [{session_label}]: {ingested} file(s) ingested");
    Ok(())
}
