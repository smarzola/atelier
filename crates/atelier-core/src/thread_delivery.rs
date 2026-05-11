use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::thread::thread_dir;
use crate::thread_events::ThreadEvent;
use crate::thread_items::ThreadItem;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DeliveryCursor {
    last_sequence: u64,
}

pub fn read_undelivered_events(
    project_path: &Path,
    thread_id: &str,
    subscriber_id: &str,
) -> Result<Vec<ThreadEvent>> {
    let cursor = read_delivery_cursor(project_path, thread_id, subscriber_id)?;
    crate::thread_events::read_thread_events(project_path, thread_id, cursor.last_sequence)
}

pub fn read_undelivered_items(
    project_path: &Path,
    thread_id: &str,
    subscriber_id: &str,
) -> Result<Vec<ThreadItem>> {
    let cursor = read_delivery_cursor(project_path, thread_id, subscriber_id)?;
    crate::thread_items::read_thread_items(project_path, thread_id, cursor.last_sequence)
}

pub fn advance_delivery_cursor(
    project_path: &Path,
    thread_id: &str,
    subscriber_id: &str,
    sequence: u64,
) -> Result<()> {
    let path = cursor_path(project_path, thread_id, subscriber_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let current = read_delivery_cursor(project_path, thread_id, subscriber_id)?;
    let cursor = DeliveryCursor {
        last_sequence: current.last_sequence.max(sequence),
    };
    fs::write(&path, serde_json::to_string_pretty(&cursor)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_delivery_cursor(
    project_path: &Path,
    thread_id: &str,
    subscriber_id: &str,
) -> Result<DeliveryCursor> {
    let path = cursor_path(project_path, thread_id, subscriber_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DeliveryCursor::default());
        }
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

fn cursor_path(project_path: &Path, thread_id: &str, subscriber_id: &str) -> std::path::PathBuf {
    let safe_subscriber_id = subscriber_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    thread_dir(project_path, thread_id)
        .join("delivery-cursors")
        .join(format!("{safe_subscriber_id}.json"))
}
