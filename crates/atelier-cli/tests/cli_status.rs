use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn status_summarizes_registered_projects_jobs_and_prompts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let job_dir = project.join(".atelier/jobs/job-status");
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("create prompts dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id":"job-status",
            "status":"waiting-for-prompt",
            "thread_id":"thread-status",
            "person":"alice",
            "dry_run":false,
            "codex_binary":"codex",
            "invocation":["app-server"]
        }))
        .expect("serialize status"),
    )
    .expect("write status");
    std::fs::write(
        prompts_dir.join("prompt-status.json"),
        r#"{
  "id": "prompt-status",
  "codex_request_id": "1",
  "method": "item/tool/requestUserInput",
  "codex_thread_id": "codex-thread-example",
  "codex_turn_id": "turn-example",
  "codex_item_id": "item-example",
  "status": "Pending",
  "summary": "Answer tool user-input prompt",
  "available_decisions": ["answer", "cancel"],
  "params": {}
}
"#,
    )
    .expect("write prompt");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Projects: 1"))
        .stdout(predicate::str::contains("Waiting prompts: 1"))
        .stdout(predicate::str::contains("Active jobs: 1"))
        .stdout(predicate::str::contains(
            "example-project\tjob-status\twaiting-for-prompt",
        ));
}

#[test]
fn prompts_inbox_lists_pending_prompts_across_registered_projects() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let job_dir = project.join(".atelier/jobs/job-inbox");
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("create prompts dir");
    std::fs::write(
        prompts_dir.join("prompt-inbox.json"),
        r#"{
  "id": "prompt-inbox",
  "codex_request_id": "1",
  "method": "item/tool/requestUserInput",
  "codex_thread_id": "codex-thread-example",
  "codex_turn_id": "turn-example",
  "codex_item_id": "item-example",
  "status": "Pending",
  "summary": "Answer project question",
  "available_decisions": ["answer", "cancel"],
  "params": {}
}
"#,
    )
    .expect("write prompt");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["prompts", "inbox"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "example-project\tjob-inbox\tprompt-inbox\tAnswer project question",
        ));
}

fn init_and_register(temp: &tempfile::TempDir, project: &std::path::Path) {
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "project",
            "init",
            project.to_str().expect("utf8 path"),
            "--name",
            "example-project",
        ])
        .assert()
        .success();
    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "projects",
            "add",
            "example-project",
            project.to_str().expect("utf8 path"),
        ])
        .assert()
        .success();
}
