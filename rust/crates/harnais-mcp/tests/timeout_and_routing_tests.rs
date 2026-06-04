//! Tests for dynamic timeout computation and model routing (D-PLAN-6).
//! ISO 25010 — Functional suitability.

use harnais_mcp::classifier::{select_model, TaskType};
use harnais_mcp::tools::compute_timeout;

#[test]
fn timeout_grows_with_prompt_length() {
    let short = compute_timeout("gemma4:31b", "short prompt");
    let long = compute_timeout("gemma4:31b", &"x".repeat(4000));
    assert!(long > short, "longer prompt should yield longer timeout");
}

#[test]
fn timeout_minimum_is_30s() {
    let t = compute_timeout("gemma3:4b", "hi");
    assert!(t.as_secs() >= 30);
}

#[test]
fn timeout_maximum_is_600s() {
    let t = compute_timeout("llama3.1:70b", &"x".repeat(100_000));
    assert!(t.as_secs() <= 600);
}

#[test]
fn slow_model_gets_longer_timeout_than_fast_model() {
    let prompt = "x".repeat(2000);
    let fast = compute_timeout("gemma3:4b", &prompt);
    let slow = compute_timeout("qwen2.5:32b-instruct-q6_K", &prompt);
    assert!(slow > fast);
}

#[test]
fn python_file_routes_to_qwen() {
    let files = vec!["skills/security-audit/run.py".to_string()];
    let model = select_model(&TaskType::Implementation, &files);
    assert!(
        model.contains("qwen2.5:32b"),
        "Python impl should route to qwen"
    );
}

#[test]
fn rust_file_routes_to_gemma4() {
    let files = vec!["src/classifier.rs".to_string()];
    let model = select_model(&TaskType::Implementation, &files);
    assert!(
        model.contains("gemma4:31b"),
        "Rust impl should route to gemma4"
    );
}

#[test]
fn boilerplate_routes_to_fast_model() {
    let model = select_model(&TaskType::Boilerplate, &[]);
    assert!(
        model.contains("gemma3:4b"),
        "Boilerplate should use fast model"
    );
}

#[test]
fn no_context_files_routes_to_qwen() {
    let model = select_model(&TaskType::Implementation, &[]);
    assert!(
        model.contains("qwen2.5:32b"),
        "Default impl should use qwen"
    );
}
