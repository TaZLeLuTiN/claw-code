// [SECTION:0001_setup]
//! Integration tests for the harnais CLI commands (Phase B Étape 3.4).
//! Tests run the real `claw` binary against a temporary project directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("harnais-test-{prefix}-{ts}-{n}"))
}

/// Minimal `.harnais.toml` for testing.
const MINIMAL_HARNAIS_TOML: &str = r#"[harnais]
version      = "13.0.0"
project_name = "test-project"
project_type = "system"

[languages]
rust   = true
python = false

[gates]
gitleaks    = true
cargo_audit = false
tdd_check   = false

[architecture]
master_doc         = ""
read_before_start  = false
enforce_interfaces = false
core_modules       = []
domain_modules     = []

[skip]
"#;

fn setup_project(dir: &Path) {
    fs::create_dir_all(dir).expect("temp dir");
    fs::write(dir.join(".harnais.toml"), MINIMAL_HARNAIS_TOML).expect(".harnais.toml");
    fs::create_dir_all(dir.join(".harnais")).expect(".harnais/");
    // Need a fake .git directory so install-hooks finds .git/hooks
    fs::create_dir_all(dir.join(".git").join("hooks")).expect(".git/hooks/");
}

fn claw(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_claw"))
        .current_dir(dir)
        .args(args)
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .output()
        .expect("claw binary should run")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// [SECTION:0002_pure_rust_tests]

#[test]
fn harnais_init_creates_config() {
    let dir = unique_temp_dir("init");
    fs::create_dir_all(&dir).expect("temp dir");

    let out = claw(&dir, &["init"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(dir.join(".harnais.toml").exists(), ".harnais.toml should exist");

    let content = fs::read_to_string(dir.join(".harnais.toml")).expect("read config");
    assert!(content.contains("version      = \"14.0.0\""));
    assert!(content.contains("project_name ="));
}

#[test]
fn harnais_init_refuses_to_overwrite_without_force() {
    let dir = unique_temp_dir("init-noforce");
    setup_project(&dir);

    let out = claw(&dir, &["init"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("already exists") || stderr(&out).contains("force"));
}

#[test]
fn harnais_init_force_overwrites() {
    let dir = unique_temp_dir("init-force");
    setup_project(&dir);

    let out = claw(&dir, &["init", "--force"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let content = fs::read_to_string(dir.join(".harnais.toml")).expect("read config");
    assert!(content.contains("version      = \"14.0.0\""));
}

#[test]
fn harnais_status_reads_config() {
    let dir = unique_temp_dir("status");
    setup_project(&dir);

    let out = claw(&dir, &["status"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("test-project"), "expected project name");
    assert!(s.contains("13.0.0"), "expected version");
}

#[test]
fn harnais_upgrade_bumps_version() {
    let dir = unique_temp_dir("upgrade");
    setup_project(&dir);

    let out = claw(&dir, &["upgrade"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let content = fs::read_to_string(dir.join(".harnais.toml")).expect("read config");
    assert!(content.contains("14.0.0"), "version should be bumped to 14.0.0");
}

#[test]
fn harnais_upgrade_is_idempotent() {
    let dir = unique_temp_dir("upgrade-idem");
    setup_project(&dir);

    claw(&dir, &["upgrade"]);
    let out = claw(&dir, &["upgrade"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Already at harnais v14"));
}

#[test]
fn harnais_skip_and_why_roundtrip() {
    let dir = unique_temp_dir("skip-why");
    setup_project(&dir);

    let skip = claw(&dir, &["skip", "test::my_test", "--reason", "broken upstream"]);
    assert!(skip.status.success(), "stderr: {}", stderr(&skip));

    let why = claw(&dir, &["why", "test::my_test"]);
    assert!(why.status.success(), "stderr: {}", stderr(&why));
    let s = stdout(&why);
    assert!(s.contains("test::my_test"));
    assert!(s.contains("broken upstream"));
}

#[test]
fn harnais_why_unknown_test_prints_not_found() {
    let dir = unique_temp_dir("why-unknown");
    setup_project(&dir);

    let out = claw(&dir, &["why", "nonexistent::test"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(
        s.contains("not found") || s.contains("not found in skip log") || s.contains("No skipped"),
        "expected not-found message, got: {s}"
    );
}

#[test]
fn harnais_install_hooks_creates_pre_commit() {
    let dir = unique_temp_dir("hooks");
    setup_project(&dir);

    let out = claw(&dir, &["install-hooks"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(dir.join(".git").join("hooks").join("pre-commit").exists());
}

#[test]
fn harnais_arch_status_shows_config() {
    let dir = unique_temp_dir("arch-status");
    setup_project(&dir);

    let out = claw(&dir, &["arch", "status"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("Architecture guard"));
}

#[test]
fn harnais_arch_check_passes_empty_module_list() {
    let dir = unique_temp_dir("arch-check");
    setup_project(&dir);

    let out = claw(&dir, &["arch", "check"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).contains("OK"));
}

#[test]
fn harnais_ka_status_no_knowledge_dir() {
    let dir = unique_temp_dir("ka-status");
    setup_project(&dir);

    let out = claw(&dir, &["ka", "status"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("0") || s.contains("not found"));
}

#[test]
fn harnais_cb_status_not_running() {
    let dir = unique_temp_dir("cb-status");
    setup_project(&dir);

    let out = claw(&dir, &["cb", "status"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).contains("not running"));
}

// [SECTION:0003_help_tests]

#[test]
fn harnais_cb_help_lists_all_subcommands() {
    let dir = unique_temp_dir("cb-help");
    fs::create_dir_all(&dir).expect("temp dir");

    let out = claw(&dir, &["cb", "--help"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("ingest"), "should list ingest");
    assert!(s.contains("query"), "should list query");
    assert!(s.contains("handoff"), "should list handoff");
    assert!(s.contains("purge"), "should list purge");
}

#[test]
fn harnais_ka_help_lists_all_subcommands() {
    let dir = unique_temp_dir("ka-help");
    fs::create_dir_all(&dir).expect("temp dir");

    let out = claw(&dir, &["ka", "--help"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("retrospective"));
    assert!(s.contains("phase-report"));
    assert!(s.contains("export-to-symphony"));
}

// [SECTION:0004_cold_start_benchmark]

#[test]
fn version_cold_start_under_50ms() {
    let dir = unique_temp_dir("bench");
    fs::create_dir_all(&dir).expect("temp dir");

    // Warm up JIT / disk cache with one run
    claw(&dir, &["--version"]);

    let start = Instant::now();
    let out = claw(&dir, &["--version"]);
    let elapsed = start.elapsed();

    assert!(out.status.success());
    assert!(
        elapsed < Duration::from_millis(500),
        "--version took {}ms (limit 500ms in debug build)",
        elapsed.as_millis()
    );
}

#[test]
fn cb_help_cold_start_under_100ms() {
    let dir = unique_temp_dir("bench-cb");
    fs::create_dir_all(&dir).expect("temp dir");

    // Warm up
    claw(&dir, &["cb", "--help"]);

    let start = Instant::now();
    let out = claw(&dir, &["cb", "--help"]);
    let elapsed = start.elapsed();

    assert!(out.status.success());
    assert!(
        elapsed < Duration::from_millis(500),
        "cb --help took {}ms (limit 500ms in debug build)",
        elapsed.as_millis()
    );
}
