use assert_cmd::Command;
use serde_json::Value;

#[test]
fn managed_work_records_protocol_and_pending_prompt() {
    let (temp, project, thread_id) = initialized_project();
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    std::fs::write(
        &fake_codex,
        r#"#!/usr/bin/env python3
import json
import sys

for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    if method == "initialize":
        print(json.dumps({"id": message["id"], "result": {"userAgent": "fake-codex/0.1", "codexHome": "/tmp/fake", "platformFamily": "unix", "platformOs": "linux"}}), flush=True)
    elif method == "initialized":
        continue
    elif method == "thread/start":
        result = {"thread": {"id": "codex-thread-example", "path": "/tmp/fake-session.jsonl"}, "model": "codex-default", "modelProvider": "fake", "cwd": message["params"]["cwd"], "instructionSources": [], "approvalPolicy": "on-request", "approvalsReviewer": "user", "sandbox": {"type": "workspaceWrite"}}
        print(json.dumps({"id": message["id"], "result": result}), flush=True)
    elif method == "turn/start":
        print(json.dumps({"id": message["id"], "result": {"turn": {"id": "turn-example", "status": "inProgress"}}}), flush=True)
        print(json.dumps({"method": "item/commandExecution/requestApproval", "id": 7, "params": {"threadId": "codex-thread-example", "turnId": "turn-example", "itemId": "call-example", "reason": "Need test approval", "command": "cargo test", "cwd": message["params"].get("cwd"), "availableDecisions": ["accept", "decline", "cancel"]}}), flush=True)
        response = json.loads(sys.stdin.readline())
        print(json.dumps({"method": "serverRequest/resolved", "params": {"threadId": "codex-thread-example", "requestId": response["id"]}}), flush=True)
        print(json.dumps({"method": "turn/completed", "params": {"threadId": "codex-thread-example", "turn": {"id": "turn-example", "status": "completed"}}}), flush=True)
        break
"#,
    )
    .expect("write fake codex");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake_codex, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }

    let assert = Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .env("PATH", prepend_to_path(&fake_bin))
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "--managed",
            "Do the managed thing",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout");
    assert!(stdout.contains("Pending prompt: prompt-7"));

    let job_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Job directory: "))
        .expect("job directory");
    let protocol =
        std::fs::read_to_string(format!("{job_dir}/protocol.jsonl")).expect("read protocol");
    assert!(protocol.contains("item/commandExecution/requestApproval"));

    let prompt: Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{job_dir}/prompts/prompt-7.json")).expect("read prompt"),
    )
    .expect("prompt json");
    assert_eq!(prompt["summary"], "Approve command: cargo test");
    assert_eq!(prompt["status"], "Pending");

    let status: Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{job_dir}/status.json")).expect("read status"),
    )
    .expect("status json");
    assert_eq!(status["status"], "waiting-for-prompt");
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

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH is set");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
