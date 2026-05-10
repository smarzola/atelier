use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMetadata {
    pub id: String,
    pub title: String,
    pub status: String,
}

pub fn create_thread(project_path: &Path, title: &str) -> Result<ThreadMetadata> {
    let thread_id = format!("thread-{}", Uuid::new_v4());
    let thread_dir = thread_dir(project_path, &thread_id);
    fs::create_dir_all(thread_dir.join("jobs")).context("create thread jobs directory")?;

    let metadata = ThreadMetadata {
        id: thread_id,
        title: title.to_string(),
        status: "active".to_string(),
    };

    let toml = toml::to_string_pretty(&metadata).context("serialize thread metadata")?;
    fs::write(thread_dir.join("thread.toml"), toml).context("write thread metadata")?;
    fs::write(thread_dir.join("summary.md"), format!("# {}\n", title))
        .context("write thread summary")?;
    fs::write(
        thread_dir.join("gateway-bindings.toml"),
        "# Gateway bindings\n",
    )
    .context("write gateway bindings")?;
    fs::write(thread_dir.join("codex-sessions.jsonl"), "").context("write codex sessions")?;

    Ok(metadata)
}

pub fn list_threads(project_path: &Path) -> Result<Vec<ThreadMetadata>> {
    let threads_dir = project_path.join(".atelier/threads");
    let mut threads = Vec::new();

    if !threads_dir.exists() {
        return Ok(threads);
    }

    for entry in fs::read_dir(&threads_dir).context("read threads directory")? {
        let entry = entry.context("read thread entry")?;
        let metadata_path = entry.path().join("thread.toml");
        if metadata_path.is_file() {
            let content = fs::read_to_string(&metadata_path).context("read thread metadata")?;
            let metadata = toml::from_str(&content).context("parse thread metadata")?;
            threads.push(metadata);
        }
    }

    threads.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(threads)
}

pub fn thread_dir(project_path: &Path, thread_id: &str) -> PathBuf {
    project_path.join(".atelier/threads").join(thread_id)
}
