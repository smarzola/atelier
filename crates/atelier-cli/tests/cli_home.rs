use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn home_init_bootstraps_home_workspace_and_registers_it() {
    let temp = tempfile::tempdir().expect("tempdir");
    let home_project = temp.path().join("atelier-home");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["home", "init", home_project.to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Atelier home"));

    assert!(home_project.join("AGENTS.md").is_file());
    assert!(
        home_project
            .join(".agents/skills/route-project/SKILL.md")
            .is_file()
    );
    assert!(
        home_project
            .join(".agents/skills/inspect-runtime/SKILL.md")
            .is_file()
    );
    assert!(home_project.join(".atelier/project.toml").is_file());

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["projects", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("home"))
        .stdout(predicate::str::contains(
            home_project.to_str().expect("utf8 path"),
        ));
}
