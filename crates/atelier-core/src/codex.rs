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
pub struct CodexResumeInvocation {
    pub binary: String,
    pub prompt: String,
    pub target: ResumeTarget,
}

#[derive(Debug, Clone)]
pub enum ResumeTarget {
    Last,
    Session(String),
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

        Ok(output_to_run_output(output))
    }
}

impl CodexResumeInvocation {
    pub fn last(prompt: String) -> Self {
        Self {
            binary: "codex".to_string(),
            prompt,
            target: ResumeTarget::Last,
        }
    }

    pub fn session(session_id: String, prompt: String) -> Self {
        Self {
            binary: "codex".to_string(),
            prompt,
            target: ResumeTarget::Session(session_id),
        }
    }

    pub fn run(&self) -> Result<CodexRunOutput> {
        let mut command = Command::new(&self.binary);
        command.args(["exec", "resume"]);
        match &self.target {
            ResumeTarget::Last => {
                command.arg("--last");
            }
            ResumeTarget::Session(session_id) => {
                command.arg(session_id);
            }
        }
        command.arg(&self.prompt);
        let output = command.output().context("run codex exec resume")?;
        Ok(output_to_run_output(output))
    }
}

fn output_to_run_output(output: std::process::Output) -> CodexRunOutput {
    CodexRunOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    }
}

pub type CodexDryRun = CodexInvocation;
