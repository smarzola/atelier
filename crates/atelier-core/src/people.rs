use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn atelier_home() -> PathBuf {
    env::var_os("ATELIER_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".atelier")))
        .unwrap_or_else(|| PathBuf::from(".atelier"))
}

pub fn add_person(id: &str) -> Result<PathBuf> {
    let memory_path = person_memory_path(id);
    if !memory_path.exists() {
        if let Some(parent) = memory_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&memory_path, format!("# Person memory: {id}\n\n"))
            .with_context(|| format!("write {}", memory_path.display()))?;
    }
    Ok(memory_path)
}

pub fn set_person_memory(id: &str, memory: &str) -> Result<PathBuf> {
    let memory_path = person_memory_path(id);
    if let Some(parent) = memory_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&memory_path, format!("# Person memory: {id}\n\n{memory}\n"))
        .with_context(|| format!("write {}", memory_path.display()))?;
    Ok(memory_path)
}

pub fn read_person_memory(id: &str) -> Result<String> {
    let memory_path = person_memory_path(id);
    match fs::read_to_string(&memory_path) {
        Ok(memory) => Ok(memory),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error).with_context(|| format!("read {}", memory_path.display())),
    }
}

fn person_memory_path(id: &str) -> PathBuf {
    atelier_home().join("people").join(id).join("memory.md")
}
