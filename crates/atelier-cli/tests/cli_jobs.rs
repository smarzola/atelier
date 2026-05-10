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
