use crate::harnais::cli::{ContextArgs, ContextCommand};
use crate::harnais::config;
use std::io::Write as IoWrite;

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: ContextArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ContextCommand::Init(a) => handle_init(a.project.as_deref()),
        ContextCommand::Start(_) => {
            Err(Box::from("context start: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
    }
}

fn handle_init(project: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
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
