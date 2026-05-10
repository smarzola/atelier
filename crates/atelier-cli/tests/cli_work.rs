use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn work_dry_run_writes_job_and_prints_codex_invocation() {
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
