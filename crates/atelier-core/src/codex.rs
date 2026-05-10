use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodexDryRun {
    pub binary: String,
    pub project_path: PathBuf,
    pub prompt: String,
}

impl CodexDryRun {
    pub fn new(project_path: &Path, prompt: String) -> Self {
        Self {
            binary: "codex".to_string(),
            project_path: project_path.to_path_buf(),
            prompt,
        }
    }

    pub fn display_command(&self) -> String {
        format!(
            "{} exec --cd {} <prompt>",
            self.binary,
            self.project_path.display()
        )
    }
}
