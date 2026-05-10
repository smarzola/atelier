use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn projects_add_records_project_in_global_registry() {
    let temp = tempfile::tempdir().expect("tempdir");
    let atelier_home = temp.path().join("atelier-home");
    let project = temp.path().join("example-project");
    std::fs::create_dir(&project).expect("create project dir");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
        .args([
            "projects",
            "add",
            "example-project",
            project.to_str().expect("utf8 path"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added project example-project"));

    let registry =
        std::fs::read_to_string(atelier_home.join("registry.toml")).expect("read registry");
    assert!(registry.contains("example-project"));
    assert!(registry.contains(project.to_str().expect("utf8 path")));
}

#[test]
fn projects_list_prints_registered_projects() {
    let temp = tempfile::tempdir().expect("tempdir");
    let atelier_home = temp.path().join("atelier-home");
    let project = temp.path().join("example-project");
    std::fs::create_dir(&project).expect("create project dir");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
        .args([
            "projects",
            "add",
            "example-project",
            project.to_str().expect("utf8 path"),
        ])
        .assert()
        .success();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
        .args(["projects", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("example-project"))
        .stdout(predicate::str::contains(
            project.to_str().expect("utf8 path"),
        ));
}
