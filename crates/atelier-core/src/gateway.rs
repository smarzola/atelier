use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::people::atelier_home;
use crate::thread::thread_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessageEvent {
    pub gateway: String,
    #[serde(default)]
    pub external_thread: Option<String>,
    #[serde(default)]
    pub external_user: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub thread: Option<String>,
    #[serde(default)]
    pub person: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonBindings {
    #[serde(default)]
    pub bindings: Vec<PersonBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonBinding {
    pub gateway: String,
    pub external_user: String,
    pub person: String,
}

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

pub fn bind_person(gateway: &str, external_user: &str, person: &str) -> Result<PersonBinding> {
    let path = person_bindings_path();
    let mut bindings = load_person_bindings(&path)?;
    let binding = PersonBinding {
        gateway: gateway.to_string(),
        external_user: external_user.to_string(),
        person: person.to_string(),
    };

    if let Some(existing) = bindings
        .bindings
        .iter_mut()
        .find(|item| item.gateway == gateway && item.external_user == external_user)
    {
        *existing = binding.clone();
    } else {
        bindings.bindings.push(binding.clone());
    }

    save_person_bindings(&path, &bindings)?;
    Ok(binding)
}

pub fn resolve_person(gateway: &str, external_user: &str) -> Result<Option<PersonBinding>> {
    let bindings = load_person_bindings(&person_bindings_path())?;
    Ok(bindings
        .bindings
        .into_iter()
        .find(|item| item.gateway == gateway && item.external_user == external_user))
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

fn person_bindings_path() -> std::path::PathBuf {
    atelier_home().join("gateway-person-bindings.toml")
}

fn bindings_path(project_path: &Path, thread_id: &str) -> std::path::PathBuf {
    thread_dir(project_path, thread_id).join("gateway-bindings.toml")
}

fn load_person_bindings(path: &Path) -> Result<PersonBindings> {
    match fs::read_to_string(path) {
        Ok(content) => {
            toml::from_str(&content).with_context(|| format!("parse {}", path.display()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(PersonBindings::default()),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

fn save_person_bindings(path: &Path, bindings: &PersonBindings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(bindings).context("serialize gateway person bindings")?;
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
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
