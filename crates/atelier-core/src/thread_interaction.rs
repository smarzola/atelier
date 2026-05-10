use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::job::JobStatus;
use crate::thread::codex_session_lineage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadInteractionDecision {
    AnswerPrompt { prompt_id: String },
    QueueForRunningJob { job_id: String },
    ContinueSession { codex_session_id: String },
    StartJob,
}

pub fn decide_thread_interaction(
    project_path: &Path,
    thread_id: &str,
    _text: &str,
) -> Result<ThreadInteractionDecision> {
    if let Some(prompt_id) = single_pending_prompt(project_path, thread_id)? {
        return Ok(ThreadInteractionDecision::AnswerPrompt { prompt_id });
    }
    if let Some(job_id) = active_job(project_path, thread_id)? {
        return Ok(ThreadInteractionDecision::QueueForRunningJob { job_id });
    }
    if let Some(codex_session_id) = latest_codex_session_id(project_path, thread_id)? {
        return Ok(ThreadInteractionDecision::ContinueSession { codex_session_id });
    }
    Ok(ThreadInteractionDecision::StartJob)
}

fn single_pending_prompt(project_path: &Path, thread_id: &str) -> Result<Option<String>> {
    let mut prompt_ids = Vec::new();
    for job_dir in job_dirs(project_path)? {
        let Some(status) = read_job_status(&job_dir)? else {
            continue;
        };
        if status.thread_id != thread_id || status.status != "waiting-for-prompt" {
            continue;
        }
        let prompts_dir = job_dir.join("prompts");
        if !prompts_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&prompts_dir).context("read prompts directory")? {
            let entry = entry.context("read prompt entry")?;
            if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            if let Some(prompt_id) = entry.path().file_stem().and_then(|value| value.to_str()) {
                prompt_ids.push(prompt_id.to_string());
            }
        }
    }
    prompt_ids.sort();
    Ok(if prompt_ids.len() == 1 {
        prompt_ids.into_iter().next()
    } else {
        None
    })
}

fn active_job(project_path: &Path, thread_id: &str) -> Result<Option<String>> {
    let mut active = Vec::new();
    for job_dir in job_dirs(project_path)? {
        let Some(status) = read_job_status(&job_dir)? else {
            continue;
        };
        if status.thread_id == thread_id
            && matches!(status.status.as_str(), "running" | "waiting-for-prompt")
        {
            active.push(status.id);
        }
    }
    active.sort();
    Ok(active.into_iter().next())
}

fn latest_codex_session_id(project_path: &Path, thread_id: &str) -> Result<Option<String>> {
    let lineage = codex_session_lineage(project_path, thread_id)?;
    let mut latest = None;
    for line in lineage.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value =
            serde_json::from_str(line).context("parse codex session lineage")?;
        if let Some(id) = value
            .get("codex_thread_id")
            .or_else(|| value.get("codex_session_id"))
            .and_then(serde_json::Value::as_str)
        {
            latest = Some(id.to_string());
        }
    }
    Ok(latest)
}

fn job_dirs(project_path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let jobs_dir = project_path.join(".atelier/jobs");
    if !jobs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in fs::read_dir(&jobs_dir).context("read jobs directory")? {
        let entry = entry.context("read job entry")?;
        if entry.path().is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn read_job_status(job_dir: &Path) -> Result<Option<JobStatus>> {
    let path = job_dir.join("status.json");
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    let status =
        serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(status))
}
