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
            "Design",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let thread_id = String::from_utf8(output).expect("utf8 stdout");
    let thread_id = thread_id.trim().to_string();

    (temp, project, thread_id)
}

#[test]
fn work_dry_run_writes_job_and_prints_codex_invocation() {
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
            "--dry-run",
            "Summarize the project",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("codex exec")
                .and(predicate::str::contains("--cd"))
                .and(predicate::str::contains(
                    project.to_str().expect("utf8 path"),
                ))
                .and(predicate::str::contains("<atelier-context>"))
                .and(predicate::str::contains("Current person: alice"))
                .and(predicate::str::contains("Summarize the project")),
        );

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    let job_dir = &job_dirs[0];
    assert!(job_dir.join("request.md").is_file());
    assert!(job_dir.join("context.md").is_file());
    assert!(job_dir.join("status.json").is_file());
}

#[test]
fn work_without_dry_run_requires_running_daemon() {
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
            "Summarize the project",
        ])
        .assert()
        .failure()
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

#[test]
fn work_help_does_not_expose_managed_flag() {
    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args(["work", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--managed").not());
}
