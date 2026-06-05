// [SECTION:0001_top_level]
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "claw",
    version,
    about = "Harnais v14 — AI-assisted development runtime"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize harnais in the current project
    Init(InitArgs),
    /// Upgrade harnais to the latest configuration
    Upgrade,
    /// Show harnais runtime status
    Status,
    /// Run the harnais test suite
    Test(TestArgs),
    /// Explain why a test is skipped
    Why(WhyArgs),
    /// Mark a test as permanently skipped
    Skip(SkipArgs),
    /// Generate a project retrospective
    Reflect(ReflectArgs),
    /// Install harnais git hooks
    #[command(name = "install-hooks")]
    InstallHooks(InstallHooksArgs),
    /// Context Broker operations
    Cb(CbArgs),
    /// Knowledge Accumulator operations
    Ka(KaArgs),
    /// Architecture guard operations
    Arch(ArchArgs),
    /// Context lifecycle operations
    Context(ContextArgs),
}

// [SECTION:0002_top_level_args]

#[derive(Args)]
pub struct InitArgs {
    /// Project name (defaults to current directory name)
    #[arg(short, long)]
    pub project: Option<String>,
    /// Force re-initialization even if .harnais.toml exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct TestArgs {
    /// Run only tests matching this pattern
    #[arg(value_name = "FILTER")]
    pub filter: Option<String>,
    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Args)]
pub struct WhyArgs {
    /// Test identifier to explain
    #[arg(value_name = "TEST_ID")]
    pub test_id: String,
}

#[derive(Args)]
pub struct SkipArgs {
    /// Test identifier to skip
    #[arg(value_name = "TEST_ID")]
    pub test_id: String,
    /// Reason for skipping
    #[arg(short, long)]
    pub reason: Option<String>,
}

#[derive(Args)]
pub struct ReflectArgs {
    /// Project name (defaults to .harnais.toml project)
    #[arg(short, long)]
    pub project: Option<String>,
    /// Milestone tag
    #[arg(short, long)]
    pub milestone: Option<String>,
    /// Number of days to look back
    #[arg(short, long, default_value = "7")]
    pub days: u32,
    /// Use Ollama for LLM-assisted reflection
    #[arg(long)]
    pub ollama: bool,
}

#[derive(Args, Clone, Copy)]
pub struct InstallHooksArgs {
    /// Force overwrite existing hooks
    #[arg(long)]
    pub force: bool,
}

// [SECTION:0003_cb_args]

#[derive(Args)]
pub struct CbArgs {
    #[command(subcommand)]
    pub command: CbCommand,
}

#[derive(Subcommand)]
pub enum CbCommand {
    /// Start the Context Broker daemon
    Start,
    /// Stop the Context Broker daemon
    Stop,
    /// Show Context Broker status
    Status,
    /// Ingest a text chunk
    Ingest(CbIngestArgs),
    /// Query recent chunks
    Query(CbQueryArgs),
    /// Handoff context to another agent
    Handoff(CbHandoffArgs),
    /// Purge all chunks for a project
    Purge(CbPurgeArgs),
}

#[derive(Args)]
pub struct CbIngestArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Content category (`architecture|constraint|config|validation|harnais|handoff|known_fix|interface`)
    #[arg(short, long, default_value = "harnais")]
    pub category: String,
    /// Text content (reads from stdin if omitted)
    #[arg(value_name = "CONTENT")]
    pub content: Option<String>,
}

#[derive(Args)]
pub struct CbQueryArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Number of days to look back
    #[arg(short, long, default_value = "7")]
    pub days: i64,
    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,
}

#[derive(Args)]
pub struct CbHandoffArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
}

#[derive(Args)]
pub struct CbPurgeArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}

// [SECTION:0004_ka_args]

#[derive(Args)]
pub struct KaArgs {
    #[command(subcommand)]
    pub command: KaCommand,
}

#[derive(Subcommand)]
pub enum KaCommand {
    /// Generate a retrospective and accumulate knowledge
    Retrospective(KaRetrospectiveArgs),
    /// Validate accumulated knowledge
    Validate(KaValidateArgs),
    /// Show knowledge accumulator status
    Status,
    /// Search knowledge entries
    Search(KaSearchArgs),
    /// Deduplicate knowledge entries
    Deduplicate,
    /// Export knowledge to Symphony format
    #[command(name = "export-to-symphony")]
    ExportToSymphony(KaExportArgs),
    /// Generate a phase completion report
    #[command(name = "phase-report")]
    PhaseReport(KaPhaseReportArgs),
}

#[derive(Args)]
pub struct KaRetrospectiveArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Milestone tag
    #[arg(short, long)]
    pub milestone: Option<String>,
    /// Number of days to look back
    #[arg(short, long, default_value = "7")]
    pub days: u32,
    /// Use Ollama for LLM-assisted retrospective
    #[arg(long)]
    pub ollama: bool,
}

#[derive(Args)]
pub struct KaValidateArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
}

#[derive(Args)]
pub struct KaSearchArgs {
    /// Search query
    #[arg(value_name = "QUERY")]
    pub query: String,
    /// Project name (searches all if omitted)
    #[arg(short, long)]
    pub project: Option<String>,
    /// Maximum number of results
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

#[derive(Args)]
pub struct KaExportArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Output file path
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct KaPhaseReportArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Phase identifier
    #[arg(value_name = "PHASE")]
    pub phase: String,
}

// [SECTION:0005_arch_args]

#[derive(Args)]
pub struct ArchArgs {
    #[command(subcommand)]
    pub command: ArchCommand,
}

#[derive(Subcommand)]
pub enum ArchCommand {
    /// Run architecture consistency checks
    Check,
    /// Show architecture guard status
    Status,
    /// Ingest an architecture document
    Ingest(ArchIngestArgs),
    /// Ingest all architecture files in the project
    #[command(name = "ingest-all")]
    IngestAll(ArchIngestAllArgs),
}

#[derive(Args)]
pub struct ArchIngestArgs {
    /// Path to the architecture document
    #[arg(value_name = "PATH")]
    pub path: std::path::PathBuf,
}

#[derive(Args)]
pub struct ArchIngestAllArgs {
    /// Root directory to scan (defaults to current directory)
    #[arg(value_name = "ROOT")]
    pub root: Option<std::path::PathBuf>,
}

// [SECTION:0006_context_args]

#[derive(Args)]
pub struct ContextArgs {
    #[command(subcommand)]
    pub command: ContextCommand,
}

#[derive(Subcommand)]
pub enum ContextCommand {
    /// Initialize a context window
    Init(ContextInitArgs),
    /// Start a context tracking session
    Start(ContextStartArgs),
}

#[derive(Args)]
pub struct ContextInitArgs {
    /// Project name
    #[arg(short, long)]
    pub project: Option<String>,
}

#[derive(Args)]
pub struct ContextStartArgs {
    /// Project name
    #[arg(short, long)]
    pub project: String,
    /// Session label
    #[arg(short, long)]
    pub label: Option<String>,
}
