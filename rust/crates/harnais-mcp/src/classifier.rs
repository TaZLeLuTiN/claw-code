//! IA task classifier — determines which provider (Claude vs Ollama) and
//! which model to use based on prompt content and target file types.
//!
//! ISO 25010:
//! - Functional (1): correct classification per heuristic rules
//! - Maintainability (7): pure function, stateless, easily testable

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Claude,
    Ollama,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Architecture,
    Design,
    Review,
    Implementation,
    Boilerplate,
    Tests,
    Distillation,
    CodeAlgo,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub task_type: TaskType,
    pub provider: Provider,
    pub model: String,
    pub confidence: f32,
    pub score_claude: i32,
    pub score_ollama: i32,
    pub reasons: Vec<String>,
}

/// Keywords indicating Claude tasks (architecture, design, review)
const CLAUDE_KEYWORDS: &[&str] = &[
    "design",
    "architecture",
    "invariant",
    "prd",
    "spec",
    "contrat",
    "stratégie",
    "strategy",
    "audit",
    "review",
    "valide",
    "validate",
    "refactor-global",
    "migration",
    "décision",
    "decision",
    "conçois",
    "conceive",
    "planifie",
    "plan",
    "scope",
    "vision",
    "philosophie",
    "philosophy",
    "debug",
    "investigate",
];

/// Keywords indicating Ollama implementation tasks
const OLLAMA_IMPL_KEYWORDS: &[&str] = &[
    "implémente",
    "implement",
    "crée",
    "create",
    "génère",
    "generate",
    "écris",
    "write",
    "build",
    "construis",
    "ajoute",
    "add",
    "refactorise",
    "refactor",
    "migre-code",
    "adapte",
    "adapt",
];

const OLLAMA_BOILERPLATE_KEYWORDS: &[&str] = &[
    "scaffold",
    "boilerplate",
    "template",
    "copie",
    "copy",
    "distille",
    "distill",
    "commit",
    "patch",
    "changelog",
    "stub",
    "skeleton",
];

const OLLAMA_TEST_KEYWORDS: &[&str] = &[
    "test",
    "tests",
    "fixture",
    "fixtures",
    "assert",
    "mock",
    "spec",
    "coverage",
    "benchmark",
];

const OLLAMA_ALGO_KEYWORDS: &[&str] = &[
    "algorithme",
    "algorithm",
    "performance",
    "perf",
    "benchmark",
    "optimise",
    "optimize",
    "complexité",
    "complexity",
    "profile",
    "flamegraph",
    "simd",
    "vectorise",
];

/// Code file extensions boosting Ollama score
const CODE_EXTENSIONS: &[&str] = &[
    ".rs", ".py", ".ts", ".js", ".go", ".java", ".c", ".cpp", ".sql", ".toml", ".sh", ".bash",
];

/// Doc file patterns boosting Claude score
const DOC_PATTERNS: &[&str] = &[
    "prd_",
    "spec_",
    "design_",
    "arch_",
    ".md",
    "claude.md",
    "readme",
    "changelog",
];

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn classify(prompt: &str, context_files: &[String]) -> ClassificationResult {
    let prompt_lower = prompt.to_lowercase();
    let mut score_claude: i32 = 0;
    let mut score_ollama: i32 = 0;
    let mut reasons = Vec::new();

    for &kw in CLAUDE_KEYWORDS {
        if prompt_lower.contains(kw) {
            score_claude += 3;
            reasons.push(format!("claude_kw:{kw}"));
        }
    }

    let mut impl_score = 0i32;
    let mut boiler_score = 0i32;
    let mut test_score = 0i32;
    let mut algo_score = 0i32;

    for &kw in OLLAMA_IMPL_KEYWORDS {
        if prompt_lower.contains(kw) {
            impl_score += 3;
            reasons.push(format!("ollama_impl_kw:{kw}"));
        }
    }
    for &kw in OLLAMA_BOILERPLATE_KEYWORDS {
        if prompt_lower.contains(kw) {
            boiler_score += 3;
            reasons.push(format!("ollama_boiler_kw:{kw}"));
        }
    }
    for &kw in OLLAMA_TEST_KEYWORDS {
        if prompt_lower.contains(kw) {
            test_score += 3;
            reasons.push(format!("ollama_test_kw:{kw}"));
        }
    }
    for &kw in OLLAMA_ALGO_KEYWORDS {
        if prompt_lower.contains(kw) {
            algo_score += 4;
            reasons.push(format!("ollama_algo_kw:{kw}"));
        }
    }

    for file in context_files {
        let file_lower = file.to_lowercase();

        for &ext in CODE_EXTENSIONS {
            if file_lower.ends_with(ext) {
                score_ollama += 2;
                reasons.push(format!("code_file:{ext}"));
                break;
            }
        }

        for &pat in DOC_PATTERNS {
            if file_lower.contains(pat) {
                score_claude += 2;
                reasons.push(format!("doc_file:{pat}"));
                break;
            }
        }
    }

    if prompt.len() > 2_000 {
        score_claude += 1;
        reasons.push("long_prompt".to_string());
    }

    let max_ollama = impl_score.max(boiler_score).max(test_score).max(algo_score);
    score_ollama += max_ollama;

    let (task_type, provider, model) = if score_claude > score_ollama {
        (
            TaskType::Architecture,
            Provider::Claude,
            "claude-sonnet-4-6".to_string(),
        )
    } else if algo_score >= max_ollama && algo_score > 0 {
        (
            TaskType::CodeAlgo,
            Provider::Ollama,
            "qwen2.5:32b-instruct-q6_K".to_string(),
        )
    } else if test_score >= boiler_score && test_score >= impl_score && test_score > 0 {
        (TaskType::Tests, Provider::Ollama, "gemma3:4b".to_string())
    } else if boiler_score >= impl_score && boiler_score > 0 {
        (
            TaskType::Boilerplate,
            Provider::Ollama,
            "gemma3:4b".to_string(),
        )
    } else if impl_score > 0 {
        (
            TaskType::Implementation,
            Provider::Ollama,
            "gemma4:31b".to_string(),
        )
    } else if score_ollama > score_claude {
        // File context boosts Ollama but no specific keyword category — standard model
        (
            TaskType::Implementation,
            Provider::Ollama,
            "gemma3:12b".to_string(),
        )
    } else {
        (
            TaskType::Unknown,
            Provider::Claude,
            "claude-sonnet-4-6".to_string(),
        )
    };

    let total = (score_claude + score_ollama) as f32;
    let confidence = if total == 0.0 {
        0.0
    } else {
        (score_claude.max(score_ollama) as f32 / total).min(1.0)
    };

    ClassificationResult {
        task_type,
        provider,
        model,
        confidence,
        score_claude,
        score_ollama,
        reasons,
    }
}
