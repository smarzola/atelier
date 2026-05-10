use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct CodexInvocation {
    pub binary: String,
    pub project_path: PathBuf,
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub struct CodexRunOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl CodexInvocation {
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

    pub fn run(&self) -> Result<CodexRunOutput> {
        let output = Command::new(&self.binary)
            .args(["exec", "--cd"])
            .arg(&self.project_path)
            .arg(&self.prompt)
            .output()
            .with_context(|| format!("run {}", self.display_command()))?;

        Ok(CodexRunOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }
}

pub type CodexDryRun = CodexInvocation;
