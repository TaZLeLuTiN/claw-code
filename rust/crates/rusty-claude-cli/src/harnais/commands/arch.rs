use crate::harnais::cli::{ArchArgs, ArchCommand};
use crate::harnais::config;

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: ArchArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ArchCommand::Check => handle_check(),
        ArchCommand::Status => handle_status(),
        ArchCommand::Ingest(_) => {
            Err(Box::from("arch ingest: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        ArchCommand::IngestAll(_) => {
            Err(Box::from("arch ingest-all: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
    }
}

fn handle_check() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_config()?;
    let root = config::find_config_root().ok_or("No .harnais.toml found")?;
    let arch = &cfg.architecture;

    let mut issues = Vec::new();

    if arch.enforce_interfaces {
        for module in &arch.core_modules {
            let path = root.join(module.trim_end_matches('/'));
            if !path.exists() {
                issues.push(format!("core module missing: {module}"));
            }
        }
        for module in &arch.domain_modules {
            let path = root.join(module.trim_end_matches('/'));
            if !path.exists() {
                issues.push(format!("domain module missing: {module}"));
            }
        }
    }

    if !arch.master_doc.is_empty() {
        let doc_path = root.join(&arch.master_doc);
        if !doc_path.exists() {
            issues.push(format!("master_doc missing: {}", arch.master_doc));
        }
    }

    if issues.is_empty() {
        println!("Architecture check: OK");
        println!("  Core modules  : {}", arch.core_modules.len());
        println!("  Domain modules: {}", arch.domain_modules.len());
    } else {
        println!("Architecture check: {} issue(s)", issues.len());
        for issue in &issues {
            println!("  ✗ {issue}");
        }
        return Err(Box::from("architecture check failed"));
    }
    Ok(())
}

fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_config()?;
    let arch = &cfg.architecture;

    println!("Architecture guard — {}", cfg.harnais.project_name);
    println!("  master_doc         : {}", arch.master_doc);
    println!("  read_before_start  : {}", arch.read_before_start);
    println!("  enforce_interfaces : {}", arch.enforce_interfaces);
    println!("  core_modules       : {}", arch.core_modules.len());
    println!("  domain_modules     : {}", arch.domain_modules.len());
    Ok(())
}
