use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub id: String,
    pub status: String,
    pub thread_id: String,
    pub person: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct CreatedJob {
    pub id: String,
    pub dir: PathBuf,
}

pub fn create_dry_run_job(
    project_path: &Path,
    thread_id: &str,
    person: &str,
    request: &str,
    context: &str,
) -> Result<CreatedJob> {
    let job_id = format!("job-{}", Uuid::new_v4());
    let job_dir = project_path.join(".atelier/jobs").join(&job_id);
    fs::create_dir_all(&job_dir).context("create job directory")?;

    fs::write(job_dir.join("request.md"), request).context("write request")?;
    fs::write(job_dir.join("context.md"), context).context("write context")?;

    let status = JobStatus {
        id: job_id.clone(),
        status: "dry-run".to_string(),
        thread_id: thread_id.to_string(),
        person: person.to_string(),
        dry_run: true,
    };
    let status_json = serde_json::to_string_pretty(&status).context("serialize job status")?;
    fs::write(job_dir.join("status.json"), status_json).context("write job status")?;

    Ok(CreatedJob {
        id: job_id,
        dir: job_dir,
    })
}
