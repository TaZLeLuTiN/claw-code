// [SECTION:0001_types]
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct HarnaisConfig {
    pub harnais: HarnaisSection,
    #[serde(default)]
    pub languages: LanguagesSection,
    #[serde(default)]
    pub gates: GatesSection,
    #[serde(default)]
    pub architecture: ArchitectureSection,
    #[serde(default)]
    pub ia: IaSection,
    #[serde(default)]
    pub context: ContextSection,
}

#[derive(Debug, Deserialize, Default)]
pub struct HarnaisSection {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub project_type: String,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Default)]
pub struct LanguagesSection {
    #[serde(default)]
    pub rust: bool,
    #[serde(default)]
    pub python: bool,
    #[serde(default)]
    pub cpp: bool,
    #[serde(default)]
    pub typescript: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Default)]
pub struct GatesSection {
    #[serde(default = "default_true")]
    pub gitleaks: bool,
    #[serde(default)]
    pub cargo_audit: bool,
    #[serde(default)]
    pub tdd_check: bool,
    #[serde(default)]
    pub pip_audit: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArchitectureSection {
    #[serde(default)]
    pub master_doc: String,
    #[serde(default)]
    pub read_before_start: bool,
    #[serde(default)]
    pub enforce_interfaces: bool,
    #[serde(default)]
    pub core_modules: Vec<String>,
    #[serde(default)]
    pub domain_modules: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct IaSection {
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_query_model")]
    pub query_model: String,
    #[serde(default = "default_retro_model")]
    pub retrospective_model: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ContextSection {
    #[serde(default)]
    pub always_load: Vec<String>,
    #[serde(default)]
    pub never_auto_load: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_query_model() -> String {
    "gemma3:4b".to_string()
}

fn default_retro_model() -> String {
    "gemma3:12b".to_string()
}

// [SECTION:0002_lookup]

/// Walk up from `start` looking for `.harnais.toml`.
pub fn find_config_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir: &Path = &cwd;
    loop {
        if dir.join(".harnais.toml").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Load and parse `.harnais.toml` from the nearest project root.
pub fn load_config() -> Result<HarnaisConfig, Box<dyn std::error::Error>> {
    let root =
        find_config_root().ok_or("No .harnais.toml found in current or parent directories")?;
    let content = std::fs::read_to_string(root.join(".harnais.toml"))?;
    Ok(toml::from_str(&content)?)
}

/// Return the `.harnais/` runtime directory for the nearest project.
pub fn harnais_runtime_dir() -> Option<PathBuf> {
    find_config_root().map(|r| r.join(".harnais"))
}
