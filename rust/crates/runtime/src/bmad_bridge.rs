//! Bridge BMAD + 7 Règles de Bristol

pub struct BmadFramework {
    pub instructions: String,
    pub prompt: String, 
    pub claude_md: String,
    pub manifest: String,
}

impl BmadFramework {
    pub fn new(project_path: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            instructions: fs::read_to_string(project_path.join(".claw-framework/INSTRUCTIONS.md"))?,
            prompt: fs::read_to_string(project_path.join(".claw-framework/PROMPT.md"))?,
            claude_md: fs::read_to_string(project_path.join("CLAUDE.md"))?,
            manifest: fs::read_to_string(project_path.join(".claw-framework/MANIFEST.md"))?,
        })
    }
    
    pub fn get_context_for_role(&self, role: &str) -> String {
        format!("{}\n\n{}\n\n{}", 
            self.instructions, 
            self.prompt.replace("[DÉFINIR RÔLE]", role),
            self.manifest
        )
    }
}
