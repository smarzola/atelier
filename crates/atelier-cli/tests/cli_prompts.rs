use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prompts_list_show_and_respond_update_pending_prompt() {
    let (temp, project, _thread_id) = initialized_project();
    let job_dir = project.join(".atelier/jobs/job-example");
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("create prompts dir");
    std::fs::write(
        prompts_dir.join("prompt-7.json"),
        r#"{
  "id": "prompt-7",
  "codex_request_id": "7",
  "method": "item/commandExecution/requestApproval",
  "codex_thread_id": "codex-thread-example",
  "codex_turn_id": "turn-example",
  "codex_item_id": "call-example",
  "status": "Pending",
  "summary": "Approve command: cargo test",
  "available_decisions": ["accept", "decline", "cancel"],
  "params": {"command": "cargo test"}
}
"#,
    )
    .expect("write prompt");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["prompts", "list", project.to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "prompt-7\tPending\tApprove command: cargo test",
        ));

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "prompts",
            "show",
            project.to_str().expect("utf8 path"),
            "prompt-7",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Method: item/commandExecution/requestApproval",
        ))
        .stdout(predicate::str::contains(
            "Decision options: accept, decline, cancel",
        ));

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "prompts",
            "respond",
            project.to_str().expect("utf8 path"),
            "prompt-7",
            "accept",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Recorded response accept for prompt-7",
        ));

    let response =
        std::fs::read_to_string(job_dir.join("responses/prompt-7.json")).expect("read response");
    assert!(response.contains("\"decision\": \"accept\""));
    let prompt = std::fs::read_to_string(prompts_dir.join("prompt-7.json")).expect("read prompt");
    assert!(prompt.contains("\"status\": \"Resolved\""));
}

#[test]
fn thread_send_approval_answers_single_pending_prompt() {
    let (temp, project, thread_id) = initialized_project();
    let job_dir = project.join(".atelier/jobs/job-example");
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("create prompts dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id":"job-example",
            "status":"waiting-for-prompt",
            "thread_id":thread_id,
            "person":"alice",
            "dry_run":false,
            "codex_binary":"codex",
            "invocation":["app-server"]
        }))
        .expect("status json"),
    )
    .expect("write status");
    std::fs::write(
        prompts_dir.join("prompt-9.json"),
        r#"{
  "id": "prompt-9",
  "codex_request_id": "9",
  "method": "item/commandExecution/requestApproval",
  "codex_thread_id": "codex-thread-example",
  "codex_turn_id": "turn-example",
  "codex_item_id": "call-example",
  "status": "Pending",
  "summary": "Approve command: cargo test",
  "available_decisions": ["accept", "decline", "cancel"],
  "params": {"command": "cargo test"}
}
"#,
    )
    .expect("write prompt");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "thread",
            "send",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "approve",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Prompt: prompt-9"));

    let response =
        std::fs::read_to_string(job_dir.join("responses/prompt-9.json")).expect("read response");
    assert!(response.contains("\"decision\": \"accept\""));

    let items = atelier_core::thread_items::read_thread_items(&project, &thread_id, 0)
        .expect("read thread items");
    assert!(
        items
            .iter()
            .any(|item| item.item_type == "atelier.approval_request"
                && item.role == "assistant"
                && item.content[0].content_type == "output_text"
                && item
                    .metadata
                    .get("prompt_id")
                    .and_then(|value| value.as_str())
                    == Some("prompt-9")),
        "pending prompt should be visible as an approval request item: {items:?}"
    );
    assert!(
        items
            .iter()
            .any(|item| item.item_type == "atelier.approval_response"
                && item.role == "user"
                && item.content[0].content_type == "input_text"
                && item.content[0].text == "approve"
                && item
                    .metadata
                    .get("prompt_id")
                    .and_then(|value| value.as_str())
                    == Some("prompt-9")),
        "thread reply should be recorded as an approval response item: {items:?}"
    );
}

#[test]
fn prompts_respond_validates_decisions_and_supports_text_payloads() {
    let (temp, project, _thread_id) = initialized_project();
    let job_dir = project.join(".atelier/jobs/job-example");
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("create prompts dir");
    std::fs::write(
        prompts_dir.join("prompt-8.json"),
        r#"{
  "id": "prompt-8",
  "codex_request_id": "8",
  "method": "item/tool/requestUserInput",
  "codex_thread_id": "codex-thread-example",
  "codex_turn_id": "turn-example",
  "codex_item_id": "tool-example",
  "status": "Pending",
  "summary": "Answer tool user-input prompt",
  "available_decisions": ["answer", "cancel"],
  "params": {"message": "Need input"}
}
"#,
    )
    .expect("write prompt");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "prompts",
            "respond",
            project.to_str().expect("utf8 path"),
            "prompt-8",
            "approve",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("choose one of: answer, cancel"));

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "prompts",
            "respond",
            project.to_str().expect("utf8 path"),
            "prompt-8",
            "answer",
            "--text",
            "example answer",
        ])
        .assert()
        .success();

    let response =
        std::fs::read_to_string(job_dir.join("responses/prompt-8.json")).expect("read response");
    assert!(response.contains("\"decision\": \"answer\""));
    assert!(response.contains("\"text\": \"example answer\""));
}

fn initialized_project() -> (tempfile::TempDir, std::path::PathBuf, String) {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
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

    let output = Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("HOME", temp.path())
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
