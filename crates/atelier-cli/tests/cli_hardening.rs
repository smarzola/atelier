use assert_cmd::Command;
use predicates::prelude::*;

fn initialized_project() -> (tempfile::TempDir, std::path::PathBuf, String) {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
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
        .args([
            "thread",
            "new",
            project.to_str().expect("utf8 path"),
            "Hardening",
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

#[test]
fn failed_work_without_daemon_does_not_create_job() {
    let (_temp, project, thread_id) = initialized_project();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "Trigger failure",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "work requires a running Atelier daemon",
        ));

    let jobs_dir = project.join(".atelier/jobs");
    let job_count = if jobs_dir.exists() {
        std::fs::read_dir(jobs_dir).expect("read jobs dir").count()
    } else {
        0
    };
    assert_eq!(job_count, 0);
}
