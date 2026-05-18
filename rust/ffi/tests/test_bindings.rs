// [SECTION:0001_helpers]
//
// Integration tests for Phase B Étape 2 ML bindings.
use pyo3::prelude::*;
// Requires: HARNAIS_EMBEDDER_FAKE=1 (avoids loading the 2 GB LaBSE model).
// Skips gracefully when HARNAIS tools/ or CB are unavailable.

static PYTHON_INIT: std::sync::Once = std::sync::Once::new();

fn ensure_python_init() -> bool {
    pyo3::prepare_freethreaded_python();

    let harnais_home = std::env::var("HARNAIS_HOME").unwrap_or_else(|_| {
        format!(
            "{}/Documents/GitHub/harnais",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    if !std::path::Path::new(&harnais_home).join("tools").exists() {
        return false;
    }

    let mut ok = true;
    PYTHON_INIT.call_once(|| {
        if harnais_ffi::runtime::init_python_runtime().is_err() {
            ok = false;
        }
    });
    ok
}

macro_rules! require_harnais {
    () => {
        if !ensure_python_init() {
            eprintln!("SKIP: HARNAIS tools/ not available");
            return;
        }
    };
}

// [SECTION:0002_embedder_tests]

#[test]
fn test_embed_single_shape_768() {
    require_harnais!();
    let vec = harnais_ffi::embedder::embed_single_internal("bonjour le monde".to_string())
        .expect("embed_single");
    assert_eq!(vec.len(), 768, "expected 768-dim LaBSE embedding");
}

#[test]
fn test_embed_single_finite_values() {
    require_harnais!();
    let vec = harnais_ffi::embedder::embed_single_internal("test".to_string())
        .expect("embed_single");
    assert!(
        vec.iter().all(|v| v.is_finite()),
        "embedding contains NaN or Inf"
    );
}

#[test]
fn test_embed_batch_shapes() {
    require_harnais!();
    let texts = vec!["hello".to_string(), "world".to_string(), "rust".to_string()];
    let batch = harnais_ffi::embedder::labse_embed_batch(texts).expect("embed_batch");
    assert_eq!(batch.len(), 3, "batch should have 3 rows");
    for row in &batch {
        assert_eq!(row.len(), 768, "each row should be 768-dim");
    }
}

#[test]
fn test_embed_single_norm_approx_1() {
    require_harnais!();
    let vec = harnais_ffi::embedder::embed_single_internal("normalisation test".to_string())
        .expect("embed_single");
    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    // LaBSE returns L2-normalized embeddings: ||v|| ≈ 1.0
    assert!(
        (norm - 1.0_f32).abs() < 0.01,
        "LaBSE embedding should be unit norm, got {norm}"
    );
}

// [SECTION:0003_coherence_tests]

#[test]
fn test_mdl_score_identical_vecs() {
    require_harnais!();
    // Two identical unit vectors → low novelty (identical content)
    let vec = harnais_ffi::embedder::embed_single_internal("identical text".to_string())
        .expect("embed for mdl");
    let score = harnais_ffi::coherence::mdl_score(vec.clone(), vec).expect("mdl_score");
    assert!(
        (0.0..=1.0).contains(&score),
        "MDL score out of [0, 1]: {score}"
    );
}

#[test]
fn test_mdl_score_different_vecs() {
    require_harnais!();
    let v1 = harnais_ffi::embedder::embed_single_internal("Rust programming language".to_string())
        .expect("embed v1");
    let v2 = harnais_ffi::embedder::embed_single_internal(
        "French cuisine boeuf bourguignon".to_string(),
    )
    .expect("embed v2");
    let score = harnais_ffi::coherence::mdl_score(v1, v2).expect("mdl_score");
    assert!(
        (0.0..=1.0).contains(&score),
        "MDL score out of [0, 1]: {score}"
    );
}

// [SECTION:0004_context_broker_tests]

#[test]
fn test_cb_ingest_and_get_recent() {
    require_harnais!();
    // Uses in-memory CB (no HARNAIS_CB_DSN → in_memory=True)
    let result = harnais_ffi::context_broker::cb_ingest_chunk(
        "test-project".to_string(),
        "Le crate harnais-ffi compile correctement avec PyO3 0.21.".to_string(),
        "known_fix".to_string(),
    )
    .expect("cb_ingest_chunk");

    pyo3::Python::with_gil(|py| {
        let action: String = result.bind(py).getattr("action").unwrap().extract().unwrap();
        assert!(
            ["CREATE_NEW", "MERGE", "DISCARD"].contains(&action.as_str()),
            "unexpected action: {action}"
        );
    });

    // Verify chunk is retrievable
    let chunks =
        harnais_ffi::context_broker::get_recent_internal("test-project".to_string(), 1, None)
            .expect("get_recent_internal");
    assert!(!chunks.is_empty(), "should retrieve at least 1 chunk");
}

#[test]
fn test_cb_get_recent_empty_project() {
    require_harnais!();
    let chunks =
        harnais_ffi::context_broker::get_recent_internal("nonexistent-project".to_string(), 7, None)
            .expect("get_recent_internal");
    // Should return empty list, not an error
    assert_eq!(chunks.len(), 0, "unknown project should return empty list");
}

// [SECTION:0005_retrospective_tests]

#[test]
fn test_retrospective_generate_no_ollama() {
    require_harnais!();
    if std::env::var("HARNAIS_CI_SKIP_NETWORK").is_ok() {
        eprintln!("SKIP: HARNAIS_CI_SKIP_NETWORK set");
        return;
    }

    // use_ollama=false → heuristic path (no HTTP calls)
    let result = harnais_ffi::retrospective::generate_internal(
        "test-project".to_string(),
        "Phase B Étape 2".to_string(),
        7,
        false,
    )
    .expect("retrospective_generate");

    assert_eq!(result.project, "test-project");
    assert!(!result.ollama_used, "expected heuristic path");
}
