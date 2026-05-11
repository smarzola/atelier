use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::thread::thread_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadItem {
    pub id: String,
    pub object: String,
    pub sequence: u64,
    #[serde(rename = "type")]
    pub item_type: String,
    pub role: String,
    pub content: Vec<ThreadItemContent>,
    pub metadata: Map<String, Value>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadItemContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

pub fn append_user_message_item(
    project_path: &Path,
    thread_id: &str,
    person: &str,
    source: &str,
    text: &str,
    metadata: Value,
) -> Result<ThreadItem> {
    let mut metadata = metadata_object(metadata);
    metadata.insert("person".to_string(), Value::String(person.to_string()));
    metadata.insert("source".to_string(), Value::String(source.to_string()));

    append_thread_item(
        project_path,
        thread_id,
        "message",
        "user",
        vec![ThreadItemContent {
            content_type: "input_text".to_string(),
            text: text.to_string(),
        }],
        metadata,
    )
}

pub fn append_assistant_message_item(
    project_path: &Path,
    thread_id: &str,
    text: &str,
    metadata: Value,
) -> Result<ThreadItem> {
    let metadata = metadata_object(metadata);

    append_thread_item(
        project_path,
        thread_id,
        "message",
        "assistant",
        vec![ThreadItemContent {
            content_type: "output_text".to_string(),
            text: text.to_string(),
        }],
        metadata,
    )
}

pub fn append_thread_item(
    project_path: &Path,
    thread_id: &str,
    item_type: &str,
    role: &str,
    content: Vec<ThreadItemContent>,
    mut metadata: Map<String, Value>,
) -> Result<ThreadItem> {
    metadata.insert("thread".to_string(), Value::String(thread_id.to_string()));
    let existing = read_thread_items(project_path, thread_id, 0)?;
    let sequence = existing.last().map(|item| item.sequence).unwrap_or(0) + 1;
    let item = ThreadItem {
        id: format!("item-{}", Uuid::new_v4()),
        object: "conversation.item".to_string(),
        sequence,
        item_type: item_type.to_string(),
        role: role.to_string(),
        content,
        metadata,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let items_path = items_path(project_path, thread_id);
    if let Some(parent) = items_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&items_path)
        .with_context(|| format!("open {}", items_path.display()))?;
    serde_json::to_writer(&mut file, &item).context("write thread item")?;
    file.write_all(b"\n").context("terminate thread item")?;
    Ok(item)
}

pub fn read_thread_items(
    project_path: &Path,
    thread_id: &str,
    after_sequence: u64,
) -> Result<Vec<ThreadItem>> {
    let items_path = items_path(project_path, thread_id);
    let content = match fs::read_to_string(&items_path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).with_context(|| format!("read {}", items_path.display())),
    };

    let mut items = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let item: ThreadItem = serde_json::from_str(line)
            .with_context(|| format!("parse thread item line {}", index + 1))?;
        if item.sequence > after_sequence {
            items.push(item);
        }
    }
    Ok(items)
}

pub fn rewrite_thread_items(
    project_path: &Path,
    thread_id: &str,
    items: &[ThreadItem],
) -> Result<()> {
    let items_path = items_path(project_path, thread_id);
    if let Some(parent) = items_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = fs::File::create(&items_path)
        .with_context(|| format!("rewrite {}", items_path.display()))?;
    for item in items {
        serde_json::to_writer(&mut file, item).context("write thread item")?;
        file.write_all(b"\n").context("terminate thread item")?;
    }
    Ok(())
}

fn items_path(project_path: &Path, thread_id: &str) -> std::path::PathBuf {
    thread_dir(project_path, thread_id).join("items.jsonl")
}

fn metadata_object(metadata: Value) -> Map<String, Value> {
    match metadata {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}
