use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn root_help_centers_thread_ux_and_hides_legacy_runtime_surfaces() {
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("thread"))
        .stdout(predicate::str::contains("\n  work").not())
        .stdout(predicate::str::contains("\n  jobs").not())
        .stdout(predicate::str::contains("\n  prompts").not());
}

#[test]
fn debug_help_contains_internal_runtime_surfaces() {
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .args(["debug", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("jobs"))
        .stdout(predicate::str::contains("prompts"))
        .stdout(predicate::str::contains("events"));
}

#[test]
fn thread_follow_help_uses_conversation_items_not_events() {
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .args(["thread", "follow", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conversation items"))
        .stdout(predicate::str::contains("events").not());
}
