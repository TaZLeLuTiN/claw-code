// [SECTION:0001_dispatch]
use crate::harnais::cli::{CbArgs, CbCommand};
use crate::harnais::config;

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: CbArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        CbCommand::Start => handle_start(),
        CbCommand::Stop => handle_stop(),
        CbCommand::Status => handle_status(),
        CbCommand::Ingest(_) => {
            Err(Box::from("cb ingest: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        CbCommand::Query(_) => {
            Err(Box::from("cb query: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        CbCommand::Handoff(_) => {
            Err(Box::from("cb handoff: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        CbCommand::Purge(a) => handle_purge(&a.project, a.yes),
    }
}

// [SECTION:0002_daemon]

fn pid_path() -> Option<std::path::PathBuf> {
    config::harnais_runtime_dir().map(|d| d.join("cb.pid"))
}

fn read_pid() -> Option<u32> {
    let path = pid_path()?;
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn is_running(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn handle_start() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(pid) = read_pid() {
        if is_running(pid) {
            println!("Context Broker already running (pid {pid})");
            return Ok(());
        }
    }

    let harnais_home = std::env::var("HARNAIS_HOME").unwrap_or_else(|_| {
        format!(
            "{}/Documents/GitHub/harnais",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    let script = std::path::Path::new(&harnais_home)
        .join("scripts")
        .join("harnais");

    let runtime_dir = config::harnais_runtime_dir()
        .ok_or("No .harnais.toml found — run `claw init` first")?;
    std::fs::create_dir_all(&runtime_dir)?;

    let log_path = runtime_dir.join("cb.log");
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)?;

    let child = std::process::Command::new(&script)
        .args(["cb", "start"])
        .stdout(log_file.try_clone()?)
        .stderr(log_file)
        .spawn()?;

    let pid = child.id();
    std::fs::write(runtime_dir.join("cb.pid"), pid.to_string())?;
    println!("Context Broker started (pid {pid})");
    println!("  Log: {}", log_path.display());
    Ok(())
}

fn handle_stop() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pid) = read_pid() else {
        println!("Context Broker is not running (no pid file)");
        return Ok(());
    };

    if !is_running(pid) {
        println!("Context Broker not running (stale pid {pid})");
        if let Some(path) = pid_path() {
            let _ = std::fs::remove_file(path);
        }
        return Ok(());
    }

    let _ = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status();
    println!("Sent SIGTERM to Context Broker (pid {pid})");

    if let Some(path) = pid_path() {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    match read_pid() {
        Some(pid) if is_running(pid) => println!("Context Broker: running (pid {pid})"),
        Some(pid) => println!("Context Broker: not running (stale pid {pid})"),
        None => println!("Context Broker: not running"),
    }

    if let Some(status_file) = config::harnais_runtime_dir().map(|d| d.join("status.json")) {
        if status_file.exists() {
            let content = std::fs::read_to_string(status_file)?;
            println!("{}", content.trim());
        }
    }
    Ok(())
}

// [SECTION:0003_purge]

fn handle_purge(project: &str, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !yes {
        println!("This will purge all Context Broker data for project '{project}'.");
        println!("Re-run with --yes to confirm.");
        return Ok(());
    }

    // PURE_RUST: remove in-memory/file-backed CB data only
    // Full DB purge requires VIA_FFI (Étape 3.3)
    let root = config::find_config_root().ok_or("No .harnais.toml found")?;
    let cb_dir = root.join(".harnais").join("cb_data").join(project);
    if cb_dir.exists() {
        std::fs::remove_dir_all(&cb_dir)?;
        println!("Purged CB data for project '{project}': {}", cb_dir.display());
    } else {
        println!("No local CB data found for project '{project}'");
    }
    Ok(())
}
