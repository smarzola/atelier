use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Missing,
    Failed,
}

impl CheckStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

impl DoctorCheck {
    fn ok(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            detail: detail.into(),
        }
    }

    fn missing(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Missing,
            detail: detail.into(),
        }
    }

    fn failed(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Failed,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn is_ok(&self) -> bool {
        self.checks
            .iter()
            .all(|check| matches!(check.status, CheckStatus::Ok))
    }
}

pub fn run_doctor(project_path: Option<&Path>) -> DoctorReport {
    let mut checks = Vec::new();

    match find_on_path("codex") {
        Some(codex_path) => {
            checks.push(DoctorCheck::ok(
                "Codex binary",
                codex_path.display().to_string(),
            ));
            checks.push(check_codex_version(&codex_path));
            checks.push(check_codex_help(&codex_path, "exec", "Codex exec"));
            checks.push(check_codex_help(&codex_path, "resume", "Codex resume"));
        }
        None => checks.push(DoctorCheck::missing(
            "Codex binary",
            "codex not found on PATH",
        )),
    }

    if let Some(project_path) = project_path {
        checks.extend(check_project(project_path));
    }

    DoctorReport { checks }
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for path in env::split_paths(&path_var) {
        if path.as_os_str().is_empty() {
            continue;
        }
        let candidate = path.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn check_codex_version(codex_path: &Path) -> DoctorCheck {
    match Command::new(codex_path).arg("--version").output() {
        Ok(output) if output.status.success() => DoctorCheck::ok(
            "Codex version",
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ),
        Ok(output) => DoctorCheck::failed(
            "Codex version",
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ),
        Err(error) => DoctorCheck::failed("Codex version", error.to_string()),
    }
}

fn check_codex_help(codex_path: &Path, subcommand: &str, name: &str) -> DoctorCheck {
    match Command::new(codex_path)
        .args([subcommand, "--help"])
        .output()
    {
        Ok(output) if output.status.success() => DoctorCheck::ok(name, "available"),
        Ok(output) => DoctorCheck::failed(
            name,
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ),
        Err(error) => DoctorCheck::failed(name, error.to_string()),
    }
}

fn check_project(project_path: &Path) -> Vec<DoctorCheck> {
    vec![
        check_path("Project path", project_path, "directory"),
        check_path(
            "Project manifest",
            &project_path.join(".atelier/project.toml"),
            "file",
        ),
        check_path(
            "Project instructions",
            &project_path.join("AGENTS.md"),
            "file",
        ),
        check_path(
            "Threads directory",
            &project_path.join(".atelier/threads"),
            "directory",
        ),
    ]
}

fn check_path(name: &str, path: &Path, expected: &str) -> DoctorCheck {
    let exists = match expected {
        "directory" => path.is_dir(),
        "file" => path.is_file(),
        _ => path.exists(),
    };

    if exists {
        DoctorCheck::ok(name, path.display().to_string())
    } else {
        DoctorCheck::missing(name, format!("missing {expected}: {}", path.display()))
    }
}
