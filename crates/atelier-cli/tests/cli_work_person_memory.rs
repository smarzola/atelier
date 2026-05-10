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
fn work_dry_run_injects_only_current_person_memory_from_global_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let atelier_home = temp.path().join("atelier-home");
    let (_project_temp, project, thread_id) = initialized_project();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
        .args([
            "people",
            "memory",
            "set",
            "alice",
            "Prefers concise updates.",
        ])
        .assert()
        .success();
    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
        .args(["people", "memory", "set", "bob", "Prefers verbose updates."])
        .assert()
        .success();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
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
        .stdout(predicates::str::contains("Person memory:"))
        .stdout(predicates::str::contains("Prefers concise updates."))
        .stdout(predicates::str::contains("Prefers verbose updates.").not());
}
