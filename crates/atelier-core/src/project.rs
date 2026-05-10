use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
}

pub fn init_project(path: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("create project directory {}", path.display()))?;

    let agents_path = path.join("AGENTS.md");
    if agents_path.exists() {
        bail!("AGENTS.md already exists; refusing to overwrite");
    }

    let atelier_dir = path.join(".atelier");
    for dir in ["inbox", "threads", "jobs", "memory", "artifacts"] {
        fs::create_dir_all(atelier_dir.join(dir))
            .with_context(|| format!("create .atelier/{dir}"))?;
    }

    fs::write(
        &agents_path,
        format!(
            "# {name}\n\nThis is an Atelier project. Project knowledge belongs in this folder.\n"
        ),
    )
    .with_context(|| format!("write {}", agents_path.display()))?;

    let metadata = ProjectMetadata {
        name: name.to_string(),
    };
    let toml = toml::to_string_pretty(&metadata).context("serialize project metadata")?;
    fs::write(atelier_dir.join("project.toml"), toml).context("write project metadata")?;

    Ok(())
}
