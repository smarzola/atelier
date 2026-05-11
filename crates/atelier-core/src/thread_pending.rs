use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingThreadInteraction {
    pub kind: String,
    pub item_id: String,
    pub job_id: String,
    pub prompt_id: String,
    pub choices: Vec<String>,
}

pub fn pending_interaction_path(project_path: &Path, thread_id: &str) -> PathBuf {
    project_path
        .join(".atelier")
        .join("threads")
        .join(thread_id)
        .join("pending.json")
}

pub fn write_pending_interaction(
    project_path: &Path,
    thread_id: &str,
    pending: &PendingThreadInteraction,
) -> Result<()> {
    let path = pending_interaction_path(project_path, thread_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(pending).context("serialize pending interaction")?;
    fs::write(&path, format!("{data}\n")).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn read_pending_interaction(
    project_path: &Path,
    thread_id: &str,
) -> Result<Option<PendingThreadInteraction>> {
    let path = pending_interaction_path(project_path, thread_id);
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let pending = serde_json::from_str(&data).context("parse pending interaction")?;
    Ok(Some(pending))
}

pub fn clear_pending_interaction(project_path: &Path, thread_id: &str) -> Result<()> {
    let path = pending_interaction_path(project_path, thread_id);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("remove {}", path.display())),
    }
}
