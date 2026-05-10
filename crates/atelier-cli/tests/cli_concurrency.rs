use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn work_refuses_second_writer_in_same_project_by_default() {
    let (temp, project, thread_id) = initialized_project();
    let existing_job = project.join(".atelier/jobs/job-active-writer");
    std::fs::create_dir_all(&existing_job).expect("create job dir");
    std::fs::write(
        existing_job.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": "job-active-writer",
            "status": "waiting-for-prompt",
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
        existing_job.join("worker.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "pid": std::process::id(),
            "idle_timeout_seconds": 300
        }))
        .expect("serialize worker"),
    )
    .expect("write worker");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "bob",
            "Second writer",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "work requires a running Atelier daemon",
        ));
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
