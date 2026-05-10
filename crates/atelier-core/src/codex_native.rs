use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub fn add_project_skill(project_path: &Path, source_skill: &Path) -> Result<String> {
    let skill_name = source_skill
        .file_name()
        .and_then(|name| name.to_str())
        .context("skill path must have a UTF-8 final component")?
        .to_string();
    let destination = project_path.join(".agents/skills").join(&skill_name);
    copy_dir_all(source_skill, &destination)?;
    Ok(skill_name)
}

pub fn add_project_mcp_server(
    project_path: &Path,
    name: &str,
    command: &str,
    args: &[String],
) -> Result<()> {
    let config_path = project_path.join(".codex/config.toml");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut config = match fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str::<CodexConfig>(&content)
            .with_context(|| format!("parse {}", config_path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => CodexConfig::default(),
        Err(error) => Err(error).with_context(|| format!("read {}", config_path.display()))?,
    };

    config.mcp_servers.insert(
        name.to_string(),
        McpServerConfig {
            command: command.to_string(),
            args: args.to_vec(),
        },
    );

    let content = toml::to_string_pretty(&config).context("serialize codex config")?;
    fs::write(&config_path, content).with_context(|| format!("write {}", config_path.display()))?;
    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CodexConfig {
    #[serde(default)]
    mcp_servers: std::collections::BTreeMap<String, McpServerConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpServerConfig {
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).with_context(|| format!("create {}", destination.display()))?;
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry.context("read skill entry")?;
        let file_type = entry.file_type().context("read skill entry type")?;
        let destination_path: PathBuf = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &destination_path)
                .with_context(|| format!("copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}
