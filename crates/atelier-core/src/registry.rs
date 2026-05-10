use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::people::atelier_home;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub projects: Vec<RegisteredProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredProject {
    pub name: String,
    pub path: PathBuf,
}

pub fn add_project(name: &str, path: &Path) -> Result<RegisteredProject> {
    let mut registry = load_registry()?;
    let project = RegisteredProject {
        name: name.to_string(),
        path: path.to_path_buf(),
    };

    if let Some(existing) = registry
        .projects
        .iter_mut()
        .find(|project| project.name == name)
    {
        *existing = project.clone();
    } else {
        registry.projects.push(project.clone());
    }

    save_registry(&registry)?;
    Ok(project)
}

pub fn list_projects() -> Result<Vec<RegisteredProject>> {
    Ok(load_registry()?.projects)
}

pub fn resolve_project_path(project: &str) -> Result<PathBuf> {
    let path = PathBuf::from(project);
    if path.components().count() == 1 {
        if let Some(registered) = load_registry()?
            .projects
            .into_iter()
            .find(|registered| registered.name == project)
        {
            return Ok(registered.path);
        }
    }
    if path.exists() || path.components().count() > 1 {
        return Ok(path);
    }
    anyhow::bail!("project alias not found: {project}")
}

fn registry_path() -> PathBuf {
    atelier_home().join("registry.toml")
}

fn load_registry() -> Result<Registry> {
    let path = registry_path();
    match fs::read_to_string(&path) {
        Ok(content) => {
            toml::from_str(&content).with_context(|| format!("parse {}", path.display()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Registry::default()),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

fn save_registry(registry: &Registry) -> Result<()> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(registry).context("serialize registry")?;
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
