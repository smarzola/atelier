use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

#[derive(Debug, Clone, Default)]
pub struct CodexPolicy {
    pub approval_policy: Option<String>,
    pub sandbox: Option<String>,
    pub model: Option<String>,
    pub search: bool,
}

impl CodexPolicy {
    pub fn args(&self, include_sandbox: bool) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(approval_policy) = &self.approval_policy {
            args.push("-c".to_string());
            args.push(format!("approval_policy=\"{approval_policy}\""));
        }
        if include_sandbox {
            if let Some(sandbox) = &self.sandbox {
                args.push("--sandbox".to_string());
                args.push(sandbox.clone());
            }
        }
        if let Some(model) = &self.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        if self.search {
            args.push("--search".to_string());
        }
        args
    }
}

#[derive(Debug, Clone)]
pub struct CodexInvocation {
    pub binary: String,
    pub project_path: PathBuf,
    pub prompt: String,
    pub policy: CodexPolicy,
}

#[derive(Debug, Clone)]
pub struct CodexResumeInvocation {
    pub binary: String,
    pub prompt: String,
    pub target: ResumeTarget,
    pub policy: CodexPolicy,
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
    pub exit_code: Option<i32>,
    pub invocation: Vec<String>,
    pub codex_binary: String,
}

impl CodexInvocation {
    pub fn new(project_path: &Path, prompt: String) -> Self {
        Self::with_policy(project_path, prompt, CodexPolicy::default())
    }

    pub fn with_policy(project_path: &Path, prompt: String, policy: CodexPolicy) -> Self {
        Self {
            binary: "codex".to_string(),
            project_path: project_path.to_path_buf(),
            prompt,
            policy,
        }
    }

    pub fn display_command(&self) -> String {
        let mut args = vec!["exec".to_string()];
        args.extend(self.policy.args(true));
        args.extend([
            "--cd".to_string(),
            self.project_path.display().to_string(),
            "<prompt>".to_string(),
        ]);
        format!("{} {}", self.binary, args.join(" "))
    }

    pub fn run(&self) -> Result<CodexRunOutput> {
        let invocation = self.invocation();
        let output = self
            .command()
            .output()
            .with_context(|| format!("run {}", self.display_command()))?;

        Ok(output_to_run_output(
            output,
            self.binary.clone(),
            invocation,
        ))
    }

    pub fn run_interactive(&self) -> Result<CodexRunOutput> {
        let invocation = self.invocation();
        let status = self
            .command()
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("run interactive {}", self.display_command()))?;

        Ok(CodexRunOutput {
            stdout: String::new(),
            stderr: String::new(),
            success: status.success(),
            exit_code: status.code(),
            invocation,
            codex_binary: self.binary.clone(),
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command.arg("exec");
        command.args(self.policy.args(true));
        command
            .arg("--cd")
            .arg(&self.project_path)
            .arg(&self.prompt);
        command
    }

    fn invocation(&self) -> Vec<String> {
        let mut invocation = vec!["exec".to_string()];
        invocation.extend(self.policy.args(true));
        invocation.extend([
            "--cd".to_string(),
            self.project_path.display().to_string(),
            "<prompt>".to_string(),
        ]);
        invocation
    }
}

impl CodexResumeInvocation {
    pub fn last(prompt: String) -> Self {
        Self::last_with_policy(prompt, CodexPolicy::default())
    }

    pub fn last_with_policy(prompt: String, policy: CodexPolicy) -> Self {
        Self {
            binary: "codex".to_string(),
            prompt,
            target: ResumeTarget::Last,
            policy,
        }
    }

    pub fn session(session_id: String, prompt: String) -> Self {
        Self::session_with_policy(session_id, prompt, CodexPolicy::default())
    }

    pub fn session_with_policy(session_id: String, prompt: String, policy: CodexPolicy) -> Self {
        Self {
            binary: "codex".to_string(),
            prompt,
            target: ResumeTarget::Session(session_id),
            policy,
        }
    }

    pub fn run(&self) -> Result<CodexRunOutput> {
        let mut command = Command::new(&self.binary);
        command.args(["exec", "resume"]);
        command.args(self.policy.args(false));
        let mut invocation = vec!["exec".to_string(), "resume".to_string()];
        invocation.extend(self.policy.args(false));
        match &self.target {
            ResumeTarget::Last => {
                command.arg("--last");
                invocation.push("--last".to_string());
            }
            ResumeTarget::Session(session_id) => {
                command.arg(session_id);
                invocation.push(session_id.clone());
            }
        }
        command.arg(&self.prompt);
        invocation.push("<prompt>".to_string());
        let output = command.output().context("run codex exec resume")?;
        Ok(output_to_run_output(
            output,
            self.binary.clone(),
            invocation,
        ))
    }
}

fn output_to_run_output(
    output: std::process::Output,
    codex_binary: String,
    invocation: Vec<String>,
) -> CodexRunOutput {
    CodexRunOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
        exit_code: output.status.code(),
        invocation,
        codex_binary,
    }
}

pub type CodexDryRun = CodexInvocation;
