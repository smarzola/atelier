use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn jobs_list_shows_multiple_managed_jobs_and_statuses() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "project",
            "init",
            project.to_str().expect("utf8 path"),
            "--name",
            "example-project",
        ])
        .assert()
        .success();

    write_status(&project, "job-alpha", "running", "thread-alpha");
    write_status(&project, "job-beta", "waiting-for-prompt", "thread-beta");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["jobs", "list", project.to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("job-alpha\trunning\tthread-alpha"))
        .stdout(predicate::str::contains(
            "job-beta\twaiting-for-prompt\tthread-beta",
        ));
}

#[test]
fn jobs_show_prints_status_paths_and_worker_logs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "project",
            "init",
            project.to_str().expect("utf8 path"),
            "--name",
            "example-project",
        ])
        .assert()
        .success();
    write_status(&project, "job-with-logs", "succeeded", "thread-alpha");
    let job_dir = project.join(".atelier/jobs/job-with-logs");
    std::fs::write(job_dir.join("worker-stdout.log"), "worker stdout\n").expect("write stdout");
    std::fs::write(job_dir.join("worker-stderr.log"), "worker stderr\n").expect("write stderr");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "jobs",
            "show",
            project.to_str().expect("utf8 path"),
            "job-with-logs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Job: job-with-logs"))
        .stdout(predicate::str::contains("Status: succeeded"))
        .stdout(predicate::str::contains("worker-stdout.log: worker stdout"))
        .stdout(predicate::str::contains("worker-stderr.log: worker stderr"));
}

fn write_status(project: &std::path::Path, job_id: &str, status: &str, thread: &str) {
    let job_dir = project.join(".atelier/jobs").join(job_id);
    std::fs::create_dir_all(&job_dir).expect("create job dir");
    std::fs::write(
        job_dir.join("status.json"),
        format!(
            r#"{{
  "id": "{job_id}",
  "status": "{status}",
  "thread_id": "{thread}",
  "person": "alice",
  "dry_run": false,
  "codex_binary": "codex",
  "invocation": ["app-server"]
}}
"#
        ),
    )
    .expect("write status");
}
