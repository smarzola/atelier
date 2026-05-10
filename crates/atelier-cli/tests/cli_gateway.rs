use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn gateway_bind_records_binding_for_thread() {
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
            "Gateway work",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let thread_id = String::from_utf8(output)
        .expect("utf8 stdout")
        .trim()
        .to_string();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "gateway",
            "bind",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--gateway",
            "telegram",
            "--external-thread",
            "chat-123:topic-456",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Bound telegram:chat-123:topic-456",
        ));

    let bindings = std::fs::read_to_string(
        project
            .join(".atelier/threads")
            .join(&thread_id)
            .join("gateway-bindings.toml"),
    )
    .expect("read bindings");
    assert!(bindings.contains("gateway = \"telegram\""));
    assert!(bindings.contains("external_thread = \"chat-123:topic-456\""));
}

#[test]
fn gateway_resolve_prints_bound_thread() {
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
            "Gateway work",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let thread_id = String::from_utf8(output)
        .expect("utf8 stdout")
        .trim()
        .to_string();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "gateway",
            "bind",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--gateway",
            "telegram",
            "--external-thread",
            "chat-123:topic-456",
        ])
        .assert()
        .success();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "gateway",
            "resolve",
            project.to_str().expect("utf8 path"),
            "--gateway",
            "telegram",
            "--external-thread",
            "chat-123:topic-456",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(thread_id));
}
