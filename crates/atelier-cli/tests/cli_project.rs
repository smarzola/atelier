use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_help_mentions_core_commands() {
    let mut cmd = Command::cargo_bin("atelier").expect("atelier binary exists");

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("project")
            .and(predicate::str::contains("thread"))
            .and(predicate::str::contains("work")),
    );
}

#[test]
fn project_init_creates_project_scaffold() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");

    let mut cmd = Command::cargo_bin("atelier").expect("atelier binary exists");
    cmd.args([
        "project",
        "init",
        project.to_str().expect("utf8 path"),
        "--name",
        "example-project",
    ])
    .assert()
    .success();

    assert!(project.join("AGENTS.md").is_file());
    assert!(project.join(".atelier/project.toml").is_file());
    assert!(project.join(".atelier/inbox").is_dir());
    assert!(project.join(".atelier/threads").is_dir());
    assert!(project.join(".atelier/jobs").is_dir());
    assert!(project.join(".atelier/memory").is_dir());
    assert!(project.join(".atelier/artifacts").is_dir());

    let project_toml =
        std::fs::read_to_string(project.join(".atelier/project.toml")).expect("read project.toml");
    assert!(project_toml.contains("name = \"example-project\""));
}
