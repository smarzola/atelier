use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::thread::thread_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuedThreadMessage {
    pub sequence: u64,
    pub person: String,
    pub text: String,
    pub status: String,
}

pub fn queue_thread_message(
    project_path: &Path,
    thread_id: &str,
    person: &str,
    text: &str,
) -> Result<QueuedThreadMessage> {
    let existing = read_queued_messages(project_path, thread_id)?;
    let sequence = existing.last().map(|message| message.sequence).unwrap_or(0) + 1;
    let message = QueuedThreadMessage {
        sequence,
        person: person.to_string(),
        text: text.to_string(),
        status: "queued".to_string(),
    };
    let path = queue_path(project_path, thread_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    serde_json::to_writer(&mut file, &message).context("write queued thread message")?;
    file.write_all(b"\n")
        .context("terminate queued thread message")?;
    Ok(message)
}

pub fn read_queued_messages(
    project_path: &Path,
    thread_id: &str,
) -> Result<Vec<QueuedThreadMessage>> {
    let path = queue_path(project_path, thread_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    let mut messages = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        messages.push(
            serde_json::from_str(line)
                .with_context(|| format!("parse queued message line {}", index + 1))?,
        );
    }
    Ok(messages)
}

pub fn mark_queued_messages_ready(
    project_path: &Path,
    thread_id: &str,
    job_id: Option<&str>,
) -> Result<()> {
    for message in read_queued_messages(project_path, thread_id)?
        .into_iter()
        .filter(|message| message.status == "queued")
    {
        crate::thread_events::append_thread_event(
            project_path,
            thread_id,
            job_id,
            "queued_message_ready",
            serde_json::json!({
                "queued_sequence": message.sequence,
                "person": message.person,
                "text": message.text,
            }),
        )?;
    }
    Ok(())
}

fn queue_path(project_path: &Path, thread_id: &str) -> std::path::PathBuf {
    thread_dir(project_path, thread_id).join("queued-messages.jsonl")
}
