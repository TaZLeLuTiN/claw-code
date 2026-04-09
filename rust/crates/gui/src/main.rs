use axum::{
    routing::{get, post},
    Router,
    extract::{Json, State, Path},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

mod ollama;
use ollama::OllamaService;

// Types de données
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub language: String,
    pub framework: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub name: String,
    pub provider: String,
    pub capabilities: Vec<String>,
    pub size: String,
    pub priority: i32,
    pub recommended_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct ModelRecommendation {
    pub role: String,
    pub primary_model: String,
    pub fallback_model: String,
    pub use_case: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecisionStatus {
    Approved,
    Rejected,
    Pending,
    Implemented,
}

// État de l'application
#[derive(Clone)]
pub struct AppState {
    pub projects: Arc<Mutex<HashMap<String, ProjectConfig>>>,
    pub ai_models: Arc<Mutex<Vec<AIModel>>>,
    pub project_bridges: Arc<Mutex<HashMap<String, Vec<String>>>>,
    pub ollama: Arc<OllamaService>,
}

// Handlers API
async fn health_check() -> &'static str {
    "OK"
}

async fn get_projects(State(state): State<AppState>) -> impl IntoResponse {
    let projects = state.projects.lock().await;
    Json(projects.values().cloned().collect::<Vec<ProjectConfig>>())
}

async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<ProjectCreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let config = ProjectConfig {
        name: payload.name.clone(),
        path: PathBuf::from(&payload.path),
        language: payload.language,
        framework: "BMAD".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    
    setup_project_structure(&config.path, &config.name)?;
    
    let mut projects = state.projects.lock().await;
    projects.insert(payload.name.clone(), config);
    
    Ok((StatusCode::CREATED, Json(ResponseMessage { message: "Projet créé avec succès".to_string() })))
}

async fn get_ai_models(State(state): State<AppState>) -> impl IntoResponse {
    let models = state.ai_models.lock().await;
    Json(models.clone())
}

async fn generate_text(
    State(state): State<AppState>,
    Json(payload): Json<GenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = state.ollama.generate(&payload.model, &payload.prompt).await?;
    Ok(Json(GenerateResponse { text: response }))
}

async fn list_ollama_models(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let models = state.ollama.list_models().await?;
    Ok(Json(models))
}

async fn get_recommendations() -> impl IntoResponse {
    let recommendations = vec![
        ModelRecommendation {
            role: "architect".to_string(),
            primary_model: "deepseek-coder-v2:latest".to_string(),
            fallback_model: "llama3.1:70b-instruct-q4_K_M".to_string(),
            use_case: "Architecture système et design patterns".to_string(),
        },
        ModelRecommendation {
            role: "developer".to_string(),
            primary_model: "deepseek-coder:6.7b".to_string(),
            fallback_model: "deepseek-coder-v2:latest".to_string(),
            use_case: "Développement quotidien et debugging".to_string(),
        },
        ModelRecommendation {
            role: "cto".to_string(),
            primary_model: "llama3.1:70b-instruct-q4_K_M".to_string(),
            fallback_model: "qwen2.5:32b-instruct-q6_K".to_string(),
            use_case: "Stratégie technique et roadmap".to_string(),
        },
        ModelRecommendation {
            role: "security".to_string(),
            primary_model: "codellama:7b".to_string(),
            fallback_model: "deepseek-coder-v2:latest".to_string(),
	    use_case: "Audit de sécurité et revue de code".to_string(),
        }
    ];
    
    Json(recommendations)
}

async fn get_models_for_role(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> impl IntoResponse {
    let models = state.ai_models.lock().await;
    let recommended: Vec<AIModel> = models.iter()
        .filter(|model| model.recommended_for.contains(&role))
        .cloned()
        .collect();
    
    Json(recommended)
}

// Structures pour les requêtes
#[derive(Debug, Deserialize)]
struct ProjectCreateRequest {
    name: String,
    path: String,
    language: String,
}

#[derive(Debug, Serialize)]
struct ResponseMessage {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Serialize)]
struct GenerateResponse {
    text: String,
}

// Erreur personnalisée
#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur: {}", self.0),
        ).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Fonctions utilitaires
fn setup_project_structure(path: &PathBuf, project_name: &str) -> Result<()> {
    fs::create_dir_all(path.join(".claw-framework"))?;
    
    let instructions = r#"# Instructions Fondamentales
## Règles absolues
- TOLÉRANCE ZÉRO RÉGRESSION
- VALIDATION ARCHITECTURE AVANT CODAGE
- STYLE PROFESSIONNEL EXIGÉ
- SÉCURITÉ MAXIMALE
## Processus
1. Lecture MANIFEST.md avant codage
2. Validation architecture IA
3. Tests avant implémentation
"#;
    
    fs::write(path.join(".claw-framework/INSTRUCTIONS.md"), instructions)?;
    
    let claude_content = format!("# Contexte du Projet {}\n\n## Structure BMAD\n- Brain: Prise de décision\n- Mind: Planification stratégique\n- Action: Implémentation\n- DNA: Architecture fondamentale\n\n## Règles Spécifiques\n- Validation croisée obligatoire\n- Documentation en temps réel\n- Tests unitaires pour chaque composant", 
        project_name);
    
    fs::write(path.join("CLAUDE.md"), claude_content)?;
    
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("tests"))?;
    fs::create_dir_all(path.join("docs"))?;
    
    Ok(())
}

// Fonction principale  
#[tokio::main]
async fn main() -> Result<()> {
    let webui_path = std::path::Path::new("/Users/mburini/Documents/GitHub/claude/claw-code/rust/crates/web-ui/dist");
    println!("📁 Serving from: {:?}", webui_path);
    println!("📁 Directory exists: {}", webui_path.exists());
    
    let port = std::env::var("CLAW_PORT")
        .unwrap_or_else(|_| "4001".to_string())
        .parse()
        .unwrap_or(4001);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let ollama_service = OllamaService::new("http://localhost:11434");
    
    let state = AppState {
        projects: Arc::new(Mutex::new(HashMap::new())),
        ai_models: Arc::new(Mutex::new(vec![
            AIModel {
                name: "deepseek-coder-v2:latest".to_string(),
                provider: "ollama".to_string(),
                capabilities: vec!["architecture".to_string(), "refactoring".to_string(), "large-context".to_string(), "code-generation".to_string()],
                size: "8.9GB".to_string(),
                priority: 10,
                recommended_for: vec!["architect".to_string(), "senior-developer".to_string(), "tech-lead".to_string()],
            },
            AIModel {
                name: "llama3.1:70b-instruct-q4_K_M".to_string(),
                provider: "ollama".to_string(),
                capabilities: vec!["planning".to_string(), "strategy".to_string(), "system-design".to_string(), "decision-making".to_string()],
                size: "42GB".to_string(),
                priority: 9,
                recommended_for: vec!["cto".to_string(), "product-manager".to_string(), "system-architect".to_string()],
            },
            AIModel {
                name: "deepseek-coder:6.7b".to_string(),
                provider: "ollama".to_string(),
                capabilities: vec!["quick-coding".to_string(), "debugging".to_string(), "completion".to_string(), "iteration".to_string()],
                size: "3.8GB".to_string(),
                priority: 8,
                recommended_for: vec!["developer".to_string(), "junior-developer".to_string(), "prototyping".to_string()],
            },
            AIModel {
                name: "mistral:7b".to_string(),
                provider: "ollama".to_string(),
                capabilities: vec!["documentation".to_string(), "explanation".to_string(), "teaching".to_string(), "writing".to_string()],
                size: "4.4GB".to_string(),
                priority: 7,
                recommended_for: vec!["technical-writer".to_string(), "educator".to_string(), "documentation".to_string()],
            },
            AIModel {
                name: "qwen2.5vl:7b".to_string(),
                provider: "ollama".to_string(),
                capabilities: vec!["multimodal".to_string(), "vision".to_string(), "image-analysis".to_string(), "diagrams".to_string()],
                size: "6.0GB".to_string(),
                priority: 6,
                recommended_for: vec!["designer".to_string(), "ux-researcher".to_string(), "visual-analysis".to_string()],
            },
            AIModel {
                name: "codellama:7b".to_string(),
                provider: "ollama".to_string(),
                capabilities: vec!["security".to_string(), "code-review".to_string(), "vulnerability".to_string(), "audit".to_string()],
                size: "3.8GB".to_string(),
                priority: 7,
                recommended_for: vec!["security-engineer".to_string(), "auditor".to_string(), "pen-tester".to_string()],
            }
        ])),
        project_bridges: Arc::new(Mutex::new(HashMap::new())),
        ollama: Arc::new(ollama_service),
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/projects", get(get_projects).post(create_project))
        .route("/api/ai-models", get(get_ai_models))
        .route("/api/ollama/generate", post(generate_text))
        .route("/api/ollama/models", get(list_ollama_models))
        .route("/api/recommendations", get(get_recommendations))
        .route("/api/models/role/:role", get(get_models_for_role))
        .nest_service("/", ServeDir::new("/Users/mburini/Documents/GitHub/claude/claw-code/rust/crates/web-ui/dist"))
        .with_state(state);

    println!("🚀 Serveur CLAW GUI démarré sur http://{}", addr);
    println!("📊 Interface disponible à: http://localhost:{}", port);
    println!("🤖 Ollama intégré sur: http://localhost:11434");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

