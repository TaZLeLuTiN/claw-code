use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about = "Claw Code Multi-Provider Standalone", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, help = "Project directory")]
    project: Option<PathBuf>,
    
    #[arg(long, help = "AI model to use")]
    model: Option<String>,
    
    #[arg(short, long, help = "Verbose output")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project with BMAD framework
    Init {
        name: String,
        language: String,
    },
    /// Chat with AI using project context
    Chat {
        message: String,
    },
    /// Configure AI models and settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Project management
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// List available models
    List,
    /// Set default model
    Set { model: String },
    /// Test connection to models
    Test,
}

#[derive(Subcommand)]
enum ProjectAction {
    /// Show project status
    Status,
    /// Update manifest
    Update { decision: String },
    /// Show project context
    Context,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        println!("🦀 Claw Code Standalone - Multi-Provider AI Assistant");
    }
    
    match cli.command {
        Commands::Init { name, language } => {
            initialize_project(&name, &language, cli.project.as_ref()).await?;
        }
        Commands::Chat { message } => {
            chat_with_context(cli.project.as_ref(), &message, cli.model.as_ref().map(|x| x.as_str())).await?;
        }
        Commands::Config { action } => {
            handle_config(action).await?;
        }
        Commands::Project { action } => {
            handle_project_action(action, cli.project.as_ref()).await?;
        }
    }
    
    Ok(())
}

async fn initialize_project(name: &str, language: &str, project_path: Option<&PathBuf>) -> Result<()> {
    println!("🚀 Initializing project: {} ({})", name, language);
    
    let base_path = project_path.unwrap_or(&PathBuf::from(".")).join(name);
    
    // Créer la structure du projet
    std::fs::create_dir_all(&base_path)?;
    std::fs::create_dir_all(base_path.join(".claw-framework"))?;
    
    // Créer les fichiers BMAD
    create_bmad_files(&base_path, language)?;
    
    // Initialiser Git
    initialize_git(&base_path)?;
    
    println!("✅ Project initialized with BMAD framework!");
    println!("📁 Location: {}", base_path.display());
    Ok(())
}

async fn chat_with_context(project_path: Option<&PathBuf>, message: &str, model: Option<&str>) -> Result<()> {
    println!("💬 Chat with AI...");
    
    let default_path = PathBuf::from(".");
    let project = project_path.unwrap_or(&default_path);
    
    // Charger le contexte BMAD
    let _context = load_project_context(project)?;
    
    // Utiliser le modèle spécifié ou le défaut
    let model_to_use = model.unwrap_or("codellama:7b");
    
    println!("🤖 Using model: {}", model_to_use);
    println!("📝 Message: {}", message);
    
    // TODO: Intégrer avec nos providers
    println!("🔄 Processing with BMAD context...");
    
    Ok(())
}

async fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::List => {
            println!("🤖 Available AI Models:");
            println!("  🧠 deepseek-coder-v2:latest (Optimization)");
            println!("  🔧 codellama:7b (Debugging)");
            println!("  🚀 deepseek-coder:6.7b (Performance)");
            println!("  📊 mistral:7b (Comparison)");
        }
        ConfigAction::Set { model } => {
            println!("⚙️ Setting default model to: {}", model);
            // TODO: Sauvegarder la config
        }
        ConfigAction::Test => {
            println!("🧪 Testing model connections...");
            // TODO: Tester les modèles
        }
    }
    Ok(())
}

async fn handle_project_action(action: ProjectAction, project_path: Option<&PathBuf>) -> Result<()> {
    let default_path = PathBuf::from(".");
    let project = project_path.unwrap_or(&default_path);
    
    match action {
        ProjectAction::Status => {
            println!("📊 Project Status:");
            let context = load_project_context(project)?;
            println!("{}", context);
        }
        ProjectAction::Update { decision } => {
            println!("📝 Updating manifest with decision: {}", decision);
            update_manifest(project, &decision)?;
        }
        ProjectAction::Context => {
            println!("📋 Project Context:");
            let context = load_project_context(project)?;
            println!("{}", context);
        }
    }
    Ok(())
}

fn create_bmad_files(project_path: &PathBuf, language: &str) -> Result<()> {
    use std::fs;
    use chrono::Utc;
    
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    
    // INSTRUCTIONS.md
    let instructions = format!(r#"# Instructions fondamentales du projet {}

## Règles absolues
- TOLÉRANCE ZÉRO RÉGRESSION
- INTERDICTION DE CODER AVANT VALIDATION ARCHITECTURE
- STYLE PROFESSIONNEL ET ACADEMIQUE
- SÉCURITÉ MAXIMALE (jamais d\'exposition de clés)

## Processus obligatoire
1. Toujours lire MANIFEST.md avant de coder
2. Valider l\'architecture avec l\'IA Architect
3. Générer tests avant implémentation

## Langage: {}
## Créé le: {}
"#, language, language, date_str);
    
    fs::write(project_path.join(".claw-framework/INSTRUCTIONS.md"), instructions)?;
    
    // PROMPT.md
    let prompt = r#"# Prompt dynamique - Rôle actuel

## Rôle: [DÉFINIR RÔLE - PM/Architecte/Dev/QA]
## Tâche: [DÉFINIR TÂCHE SPÉCIFIQUE]

## Contraintes:
- Utiliser Sequential Thinking
- Poser Clarifying Questions
- Basé sur foundation-document.md
"#;
    
    fs::write(project_path.join(".claw-framework/PROMPT.md"), prompt)?;
    
    // MANIFEST.md
    let manifest = format!(r#"# Manifest du projet - Historique immuable

## État actuel: Projet initialisé
## Langage: {}
## Créé le: {}

## Fonctionnalités:
- [ ] Structure de base créée

## Décisions architecturales:
- [{}] - Initialisation du projet avec framework BMAD

## Prochaines étapes:
- [ ] Configurer le langage spécifique
- [ ] Définir l'architecture initiale
- [ ] Créer les premiers tests
"#, language, language, datetime_str);
    
    fs::write(project_path.join(".claw-framework/MANIFEST.md"), manifest)?;
    
    Ok(())
}

fn load_project_context(project_path: &PathBuf) -> Result<String> {
    use std::fs;
    
    let framework_dir = project_path.join(".claw-framework");
    let instructions = fs::read_to_string(framework_dir.join("INSTRUCTIONS.md")).unwrap_or_else(|_| "# No instructions found".to_string());
    let manifest = fs::read_to_string(framework_dir.join("MANIFEST.md")).unwrap_or_else(|_| "# No manifest found".to_string());
    
    Ok(format!("{}\n\n{}", instructions, manifest))
}

fn update_manifest(project_path: &PathBuf, decision: &str) -> Result<()> {
    use std::fs;
    use chrono::Utc;
    
    let manifest_path = project_path.join(".claw-framework/MANIFEST.md");
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    let new_entry = format!("- [{}] - {}\n", timestamp, decision);
    
    let mut manifest = fs::read_to_string(&manifest_path)?;
    manifest.push_str(&new_entry);
    
    fs::write(&manifest_path, manifest)?;
    
    println!("✅ Manifest updated with decision: {}", decision);
    Ok(())
}

fn initialize_git(project_path: &PathBuf) -> Result<()> {
    use std::process::Command;
    
    let output = Command::new("git")
        .args(&["init"])
        .current_dir(project_path)
        .output()?;
    
    if output.status.success() {
        println!("✅ Git repository initialized");
    } else {
        println!("⚠️ Git initialization failed");
    }
    
    Ok(())
}
