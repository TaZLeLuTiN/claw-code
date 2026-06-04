//! Unit tests for the IA classifier.
//! ISO 25010 — Functional suitability.

use harnais_mcp::classifier::{classify, Provider, TaskType};

#[test]
fn architecture_prompt_routes_to_claude() {
    let r = classify("Conçois l'architecture du module de routage IA", &[]);
    assert_eq!(r.provider, Provider::Claude);
}

#[test]
fn design_prompt_routes_to_claude() {
    let r = classify("Design the invariants for the trust gate", &[]);
    assert_eq!(r.provider, Provider::Claude);
}

#[test]
fn rust_implementation_routes_to_ollama_complex() {
    let r = classify(
        "Implémente la commande status en Rust",
        &["src/cli.rs".to_string()],
    );
    assert_eq!(r.provider, Provider::Ollama);
    assert_eq!(r.model, "gemma4:31b");
    assert!(matches!(r.task_type, TaskType::Implementation));
}

#[test]
fn test_generation_routes_to_ollama_fast() {
    let r = classify("Écris les tests unitaires pour le module gates", &[]);
    assert_eq!(r.provider, Provider::Ollama);
    assert_eq!(r.model, "gemma3:4b");
    assert!(matches!(r.task_type, TaskType::Tests));
}

#[test]
fn boilerplate_routes_to_ollama_fast() {
    let r = classify("Génère le scaffold pour le nouveau module skills", &[]);
    assert_eq!(r.provider, Provider::Ollama);
    assert_eq!(r.model, "gemma3:4b");
    assert!(matches!(r.task_type, TaskType::Boilerplate));
}

#[test]
fn algo_prompt_routes_to_qwen() {
    let r = classify(
        "Optimise l'algorithme de recherche vectorielle (performance)",
        &[],
    );
    assert_eq!(r.provider, Provider::Ollama);
    assert_eq!(r.model, "qwen2.5:32b-instruct-q6_K");
    assert!(matches!(r.task_type, TaskType::CodeAlgo));
}

#[test]
fn code_file_context_boosts_ollama() {
    let files = vec![
        "src/main.py".to_string(),
        "tests/test_bridge.py".to_string(),
    ];
    let r = classify("Complète l'implémentation", &files);
    assert_eq!(r.provider, Provider::Ollama);
    assert!(r.score_ollama > r.score_claude);
}

#[test]
fn prd_file_context_boosts_claude() {
    let files = vec!["PRD_HARNAIS_V14_SKILLS.md".to_string()];
    let r = classify("Analyse ce document", &files);
    assert_eq!(r.provider, Provider::Claude);
}

#[test]
fn confidence_is_between_0_and_1() {
    let r = classify("Implémente quelque chose", &[]);
    assert!((0.0..=1.0).contains(&r.confidence));
}

#[test]
fn unknown_prompt_falls_back_to_claude() {
    let r = classify("xyz abc 123", &[]);
    assert_eq!(r.provider, Provider::Claude);
}
