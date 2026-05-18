use crate::harnais::cli::{KaArgs, KaCommand};
use crate::harnais::config;

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: KaArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        KaCommand::Retrospective(a) => {
            Err(Box::from(format!(
                "ka retrospective (project={}, ollama={}): not yet implemented (VIA_FFI — Étape 3.3)",
                a.project, a.ollama
            )))
        }
        KaCommand::Validate(a) => handle_validate(&a.project),
        KaCommand::Status => handle_status(),
        KaCommand::Search(_) => {
            Err(Box::from("ka search: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        KaCommand::Deduplicate => {
            Err(Box::from("ka deduplicate: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        KaCommand::ExportToSymphony(a) => handle_export(&a.project, a.output.as_deref()),
        KaCommand::PhaseReport(_) => {
            Err(Box::from("ka phase-report: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
    }
}

fn pending_path(project: &str) -> Option<std::path::PathBuf> {
    config::find_config_root().map(|r| {
        r.join("knowledge")
            .join(project)
            .join("KNOWLEDGE_PENDING.jsonl")
    })
}

fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    let root = config::find_config_root()
        .ok_or("No .harnais.toml found")?;
    let ka_dir = root.join("knowledge");

    if !ka_dir.exists() {
        println!("Knowledge accumulator: no entries (knowledge/ not found)");
        return Ok(());
    }

    let mut total = 0usize;
    let mut projects: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&ka_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let pending = entry.path().join("KNOWLEDGE_PENDING.jsonl");
        if pending.exists() {
            let count = std::fs::read_to_string(&pending)?
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count();
            total += count;
            projects.push(format!(
                "  {} — {} pending",
                entry.file_name().to_string_lossy(),
                count
            ));
        }
    }

    println!("Knowledge accumulator: {total} total pending entry/entries");
    for p in &projects {
        println!("{p}");
    }
    Ok(())
}

fn handle_validate(project: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = pending_path(project).ok_or("No .harnais.toml found")?;

    if !path.exists() {
        println!("No pending knowledge for project '{project}'");
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)?;
    let mut ok = 0usize;
    let mut errors = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(_) => ok += 1,
            Err(e) => errors.push(format!("line {}: {e}", i + 1)),
        }
    }

    if errors.is_empty() {
        println!("Validated {ok} entries for '{project}': OK");
    } else {
        println!("Validation errors for '{project}':");
        for e in &errors {
            println!("  ✗ {e}");
        }
        return Err(Box::from(format!("{} validation error(s)", errors.len())));
    }
    Ok(())
}

fn handle_export(
    project: &str,
    output: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = pending_path(project).ok_or("No .harnais.toml found")?;

    if !src.exists() {
        return Err(Box::from(format!(
            "No pending knowledge for project '{project}'"
        )));
    }

    let dest = output.map_or_else(
        || {
            src.parent()
                .unwrap_or(std::path::Path::new("."))
                .join("SYMPHONY_EXPORT.jsonl")
        },
        std::path::Path::to_path_buf,
    );

    std::fs::copy(&src, &dest)?;
    println!("Exported {} → {}", src.display(), dest.display());
    Ok(())
}
