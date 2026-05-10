use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::thread::thread_dir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayBindings {
    #[serde(default)]
    pub bindings: Vec<GatewayBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayBinding {
    pub gateway: String,
    pub external_thread: String,
    pub thread_id: String,
}

pub fn bind_thread(
    project_path: &Path,
    thread_id: &str,
    gateway: &str,
    external_thread: &str,
) -> Result<GatewayBinding> {
    let path = bindings_path(project_path, thread_id);
    let mut bindings = load_bindings(&path)?;
    let binding = GatewayBinding {
        gateway: gateway.to_string(),
        external_thread: external_thread.to_string(),
        thread_id: thread_id.to_string(),
    };

    if let Some(existing) = bindings
        .bindings
        .iter_mut()
        .find(|item| item.gateway == gateway && item.external_thread == external_thread)
    {
        *existing = binding.clone();
    } else {
        bindings.bindings.push(binding.clone());
    }

    save_bindings(&path, &bindings)?;
    Ok(binding)
}

pub fn resolve_thread(
    project_path: &Path,
    gateway: &str,
    external_thread: &str,
) -> Result<Option<GatewayBinding>> {
    let threads_dir = project_path.join(".atelier/threads");
    if !threads_dir.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(&threads_dir).context("read threads directory")? {
        let entry = entry.context("read thread entry")?;
        let path = entry.path().join("gateway-bindings.toml");
        let bindings = load_bindings(&path)?;
        if let Some(binding) = bindings
            .bindings
            .into_iter()
            .find(|item| item.gateway == gateway && item.external_thread == external_thread)
        {
            return Ok(Some(binding));
        }
    }

    Ok(None)
}

fn bindings_path(project_path: &Path, thread_id: &str) -> std::path::PathBuf {
    thread_dir(project_path, thread_id).join("gateway-bindings.toml")
}

fn load_bindings(path: &Path) -> Result<GatewayBindings> {
    match fs::read_to_string(path) {
        Ok(content) => {
            toml::from_str(&content).with_context(|| format!("parse {}", path.display()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(GatewayBindings::default())
        }
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

fn save_bindings(path: &Path, bindings: &GatewayBindings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(bindings).context("serialize gateway bindings")?;
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
