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

pub fn create_job(
    project_path: &Path,
    thread_id: &str,
    person: &str,
    request: &str,
    context: &str,
    dry_run: bool,
) -> Result<CreatedJob> {
    let job_id = format!("job-{}", Uuid::new_v4());
    let job_dir = project_path.join(".atelier/jobs").join(&job_id);
    fs::create_dir_all(&job_dir).context("create job directory")?;

    fs::write(job_dir.join("request.md"), request).context("write request")?;
    fs::write(job_dir.join("context.md"), context).context("write context")?;

    write_status(
        &job_dir,
        JobStatus {
            id: job_id.clone(),
            status: if dry_run { "dry-run" } else { "running" }.to_string(),
            thread_id: thread_id.to_string(),
            person: person.to_string(),
            dry_run,
        },
    )?;

    Ok(CreatedJob {
        id: job_id,
        dir: job_dir,
    })
}

pub fn create_dry_run_job(
    project_path: &Path,
    thread_id: &str,
    person: &str,
    request: &str,
    context: &str,
) -> Result<CreatedJob> {
    create_job(project_path, thread_id, person, request, context, true)
}

pub fn complete_job(job: &CreatedJob, thread_id: &str, person: &str, success: bool) -> Result<()> {
    write_status(
        &job.dir,
        JobStatus {
            id: job.id.clone(),
            status: if success { "succeeded" } else { "failed" }.to_string(),
            thread_id: thread_id.to_string(),
            person: person.to_string(),
            dry_run: false,
        },
    )
}

fn write_status(job_dir: &Path, status: JobStatus) -> Result<()> {
    let status_json = serde_json::to_string_pretty(&status).context("serialize job status")?;
    fs::write(job_dir.join("status.json"), status_json).context("write job status")?;
    Ok(())
}
