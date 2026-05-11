use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn thread_new_creates_thread_and_threads_list_shows_it() {
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
            "Release preparation",
            "--porcelain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("thread-"))
        .get_output()
        .stdout
        .clone();

    let thread_id = String::from_utf8(output).expect("utf8 stdout");
    let thread_id = thread_id.trim();
    let thread_dir = project.join(".atelier/threads").join(thread_id);

    assert!(thread_dir.join("thread.toml").is_file());
    assert!(thread_dir.join("summary.md").is_file());
    assert!(thread_dir.join("gateway-bindings.toml").is_file());
    assert!(thread_dir.join("codex-sessions.jsonl").is_file());
    assert!(thread_dir.join("jobs").is_dir());

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args(["threads", "list", project.to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("Release preparation"));
}
