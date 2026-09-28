//! Tests for model routing (D-PLAN-6). The dynamic timeout (hard-coded speeds) was removed — see
//! `flux_ollama_tests.rs` : a generation is abandoned on SILENCE, never predicted.
//! ISO 25010 — Functional suitability.

use harnais_mcp::classifier::{select_model, TaskType};
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
