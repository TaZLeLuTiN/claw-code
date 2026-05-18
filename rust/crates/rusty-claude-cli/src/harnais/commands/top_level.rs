// [SECTION:0001_imports]
use crate::harnais::cli::{
    InitArgs, InstallHooksArgs, ReflectArgs, SkipArgs, TestArgs, WhyArgs,
};
use crate::harnais::config;
use std::io::{BufRead, Write as IoWrite};

// [SECTION:0002_init]

pub fn handle_init(args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join(".harnais.toml");

    if config_path.exists() && !args.force {
        return Err(Box::from(
            ".harnais.toml already exists; use --force to reinitialize",
        ));
    }

    let project_name = args.project.unwrap_or_else(|| {
        cwd.file_name()
            .map_or_else(|| "unnamed".to_string(), |n| n.to_string_lossy().into_owned())
    });

    let content = format!(
        r#"# .harnais.toml — projet (Harnais v14.0.0)

[harnais]
version      = "14.0.0"
project_name = "{project_name}"
project_type = "system"

[languages]
python     = false
rust       = true
cpp        = false
typescript = false

[gates]
gitleaks    = true
cargo_audit = true
tdd_check   = true

[skip]
"#
    );

    std::fs::write(&config_path, content)?;
    std::fs::create_dir_all(cwd.join(".harnais"))?;
    println!("Initialized harnais v14 project: {project_name}");
    println!("  Config : {}", config_path.display());
    println!("  Runtime: {}/.harnais/", cwd.display());
    Ok(())
}

// [SECTION:0003_upgrade]

pub fn handle_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    let root = config::find_config_root()
        .ok_or("No .harnais.toml found — run `claw init` first")?;
    let config_path = root.join(".harnais.toml");

    let content = std::fs::read_to_string(&config_path)?;
    let current = config::load_config()?;

    if current.harnais.version.starts_with("14.") {
        println!("Already at harnais v14 ({})", current.harnais.version);
        return Ok(());
    }

    let updated = content.replace(
        &format!(r#"version      = "{}""#, current.harnais.version),
        r#"version      = "14.0.0""#,
    );
    std::fs::write(&config_path, updated)?;
    println!(
        "Upgraded {} → 14.0.0",
        current.harnais.version
    );
    Ok(())
}

// [SECTION:0004_status]

pub fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_config()?;
    println!("Project  : {}", cfg.harnais.project_name);
    println!("Version  : {}", cfg.harnais.version);
    println!("Type     : {}", cfg.harnais.project_type);

    let langs: Vec<&str> = [
        cfg.languages.rust.then_some("rust"),
        cfg.languages.python.then_some("python"),
        cfg.languages.cpp.then_some("cpp"),
        cfg.languages.typescript.then_some("typescript"),
    ]
    .iter()
    .flatten()
    .copied()
    .collect();
    println!("Languages: {}", langs.join(", "));

    let gates: Vec<&str> = [
        cfg.gates.gitleaks.then_some("gitleaks"),
        cfg.gates.cargo_audit.then_some("cargo_audit"),
        cfg.gates.tdd_check.then_some("tdd_check"),
        cfg.gates.pip_audit.then_some("pip_audit"),
    ]
    .iter()
    .flatten()
    .copied()
    .collect();
    println!("Gates    : {}", gates.join(", "));

    if let Some(runtime_dir) = config::harnais_runtime_dir() {
        let status_file = runtime_dir.join("status.json");
        if status_file.exists() {
            let status = std::fs::read_to_string(status_file)?;
            println!("\nRuntime status:\n{}", status.trim());
        } else {
            println!("\nRuntime: not running");
        }

        let skip_log = runtime_dir.join("skip_log.jsonl");
        if skip_log.exists() {
            let count = std::fs::read_to_string(skip_log)?
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count();
            println!("Skipped tests: {count}");
        }
    }
    Ok(())
}

// [SECTION:0005_test]

#[allow(clippy::needless_pass_by_value)]
pub fn handle_test(args: TestArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_config()?;
    let root = config::find_config_root()
        .ok_or("No .harnais.toml found")?;

    let mut ran = false;

    if cfg.languages.rust {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("test").current_dir(&root);
        if let Some(ref filter) = args.filter {
            cmd.arg(filter);
        }
        if args.verbose {
            cmd.arg("--").arg("--nocapture");
        }
        let status = cmd.status()?;
        if !status.success() {
            return Err(Box::from("cargo test failed"));
        }
        ran = true;
    }

    if cfg.languages.python {
        let mut cmd = std::process::Command::new("python3");
        cmd.args(["-m", "pytest"]).current_dir(&root);
        if let Some(ref filter) = args.filter {
            cmd.arg("-k").arg(filter);
        }
        if args.verbose {
            cmd.arg("-v");
        }
        let status = cmd.status()?;
        if !status.success() {
            return Err(Box::from("pytest failed"));
        }
        ran = true;
    }

    if !ran {
        println!("No test runners configured in .harnais.toml [languages] section");
    }
    Ok(())
}

// [SECTION:0006_skip_log]

#[allow(clippy::needless_pass_by_value)]
pub fn handle_why(args: WhyArgs) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = config::harnais_runtime_dir()
        .ok_or("No .harnais.toml found — run `claw init` first")?;
    let skip_log = runtime.join("skip_log.jsonl");

    if !skip_log.exists() {
        println!("No skipped tests recorded.");
        return Ok(());
    }

    let file = std::fs::File::open(&skip_log)?;
    let reader = std::io::BufReader::new(file);
    let mut found = false;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: serde_json::Value = serde_json::from_str(&line)?;
        if entry.get("test_id").and_then(|v| v.as_str()) == Some(&args.test_id) {
            let reason = entry
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("(no reason given)");
            let skipped_at = entry
                .get("skipped_at")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            println!("Test   : {}", args.test_id);
            println!("Reason : {reason}");
            println!("Skipped: {skipped_at}");
            found = true;
            break;
        }
    }

    if !found {
        println!("Test '{}' not found in skip log.", args.test_id);
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn handle_skip(args: SkipArgs) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = config::harnais_runtime_dir()
        .ok_or("No .harnais.toml found — run `claw init` first")?;
    std::fs::create_dir_all(&runtime)?;
    let skip_log = runtime.join("skip_log.jsonl");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let entry = serde_json::json!({
        "test_id": args.test_id,
        "reason": args.reason.as_deref().unwrap_or(""),
        "skipped_at": now,
    });

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&skip_log)?;
    writeln!(file, "{entry}")?;
    println!("Skipped: {}", args.test_id);
    Ok(())
}

// [SECTION:0007_install_hooks]

pub fn handle_install_hooks(args: InstallHooksArgs) -> Result<(), Box<dyn std::error::Error>> {
    let root = config::find_config_root()
        .ok_or("No .harnais.toml found — run `claw init` first")?;
    let hooks_dir = root.join(".git").join("hooks");

    if !hooks_dir.exists() {
        return Err(Box::from("No .git/hooks/ directory found — is this a git repository?"));
    }

    let harnais_home = std::env::var("HARNAIS_HOME").unwrap_or_else(|_| {
        format!(
            "{}/Documents/GitHub/harnais",
            std::env::var("HOME").unwrap_or_default()
        )
    });

    let template_src = std::path::Path::new(&harnais_home)
        .join("hooks")
        .join("pre-commit.template");
    let dest = hooks_dir.join("pre-commit");

    if dest.exists() && !args.force {
        return Err(Box::from(
            ".git/hooks/pre-commit already exists; use --force to overwrite",
        ));
    }

    if template_src.exists() {
        std::fs::copy(&template_src, &dest)?;
    } else {
        // Generate a minimal fallback hook
        let minimal = "#!/usr/bin/env bash\nset -eu\nclaw test\n";
        std::fs::write(&dest, minimal)?;
    }

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }

    println!("Installed pre-commit hook → {}", dest.display());
    Ok(())
}

// [SECTION:0008_reflect_stub]

pub fn handle_reflect(_args: ReflectArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais reflect: not yet implemented (VIA_FFI — Étape 3.3)"))
}
