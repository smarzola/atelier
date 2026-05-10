use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn people_add_creates_person_memory_file_in_atelier_home() {
    let temp = tempfile::tempdir().expect("tempdir");
    let atelier_home = temp.path().join("atelier-home");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("ATELIER_HOME", &atelier_home)
        .args(["people", "add", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added person alice"));

    let memory_path = atelier_home.join("people/alice/memory.md");
    assert!(memory_path.is_file());
    assert_eq!(
        std::fs::read_to_string(memory_path).expect("read memory"),
        "# Person memory: alice\n\n"
    );
}

#[test]
fn people_memory_set_replaces_person_memory_body() {
    let temp = tempfile::tempdir().expect("tempdir");
    let atelier_home = temp.path().join("atelier-home");

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
        .success()
        .stdout(predicate::str::contains("Updated memory for alice"));

    let memory_path = atelier_home.join("people/alice/memory.md");
    assert_eq!(
        std::fs::read_to_string(memory_path).expect("read memory"),
        "# Person memory: alice\n\nPrefers concise updates.\n"
    );
}
