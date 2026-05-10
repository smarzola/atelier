use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn jobs_list_marks_running_job_with_dead_worker_as_worker_lost() {
    let (_temp, project, thread_id) = initialized_project();
    let job_dir = project.join(".atelier/jobs/job-dead-worker");
    std::fs::create_dir_all(&job_dir).expect("create job dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": "job-dead-worker",
            "status": "running",
            "thread_id": thread_id,
            "person": "alice",
            "dry_run": false,
            "codex_binary": "codex",
            "invocation": ["app-server"]
        }))
        .expect("serialize status"),
    )
    .expect("write status");
    std::fs::write(
        job_dir.join("worker.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "pid": definitely_missing_pid(),
            "idle_timeout_seconds": 300
        }))
        .expect("serialize worker"),
    )
    .expect("write worker");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .args(["jobs", "list", project.to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("job-dead-worker\tworker-lost"));

    let status: Value = serde_json::from_str(
        &std::fs::read_to_string(job_dir.join("status.json")).expect("read status"),
    )
    .expect("status json");
    assert_eq!(status["status"], "worker-lost");
}

fn definitely_missing_pid() -> u32 {
    4_000_000_000
}

fn initialized_project() -> (tempfile::TempDir, std::path::PathBuf, String) {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
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

    let output = Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("HOME", temp.path())
        .args([
            "thread",
            "new",
            project.to_str().expect("utf8 path"),
            "Design",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let thread_id = String::from_utf8(output).expect("utf8 stdout");

    (temp, project, thread_id.trim().to_string())
}
