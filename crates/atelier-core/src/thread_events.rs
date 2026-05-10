use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::thread::thread_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadEvent {
    pub sequence: u64,
    pub timestamp_unix_seconds: u64,
    pub thread_id: String,
    pub job_id: Option<String>,
    pub kind: String,
    pub payload: Value,
}

pub fn append_thread_event(
    project_path: &Path,
    thread_id: &str,
    job_id: Option<&str>,
    kind: &str,
    payload: Value,
) -> Result<ThreadEvent> {
    let existing = read_thread_events(project_path, thread_id, 0)?;
    let sequence = existing.last().map(|event| event.sequence).unwrap_or(0) + 1;
    let event = ThreadEvent {
        sequence,
        timestamp_unix_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        thread_id: thread_id.to_string(),
        job_id: job_id.map(ToString::to_string),
        kind: kind.to_string(),
        payload,
    };

    let events_path = thread_dir(project_path, thread_id).join("events.jsonl");
    if let Some(parent) = events_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&events_path)
        .with_context(|| format!("open {}", events_path.display()))?;
    serde_json::to_writer(&mut file, &event).context("write thread event")?;
    file.write_all(b"\n").context("terminate thread event")?;
    Ok(event)
}

pub fn read_thread_events(
    project_path: &Path,
    thread_id: &str,
    after_sequence: u64,
) -> Result<Vec<ThreadEvent>> {
    let events_path = thread_dir(project_path, thread_id).join("events.jsonl");
    let content = match fs::read_to_string(&events_path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).with_context(|| format!("read {}", events_path.display())),
    };

    let mut events = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: ThreadEvent = serde_json::from_str(line)
            .with_context(|| format!("parse thread event line {}", index + 1))?;
        if event.sequence > after_sequence {
            events.push(event);
        }
    }
    Ok(events)
}
