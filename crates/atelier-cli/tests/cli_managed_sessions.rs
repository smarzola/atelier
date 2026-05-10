use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn managed_work_records_codex_thread_start_metadata_in_thread_lineage() {
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
    if message.get("method") == "initialize":
        print(json.dumps({"id": message["id"], "result": {"userAgent": "fake-codex/0.1", "codexHome": "/tmp/fake", "platformFamily": "unix", "platformOs": "linux"}}), flush=True)
    elif message.get("method") == "initialized":
        continue
    elif message.get("method") == "thread/start":
        print(json.dumps({"id": message["id"], "result": {"thread": {"id": "codex-thread-lineage", "path": "/tmp/fake-session-lineage.jsonl"}, "model": "codex-default", "modelProvider": "fake", "cwd": message["params"]["cwd"], "instructionSources": [], "approvalPolicy": "on-request", "approvalsReviewer": "user", "sandbox": {"type": "workspaceWrite"}}}), flush=True)
    elif message.get("method") == "turn/start":
        print(json.dumps({"id": message["id"], "result": {"turn": {"id": "turn-lineage", "status": "completed"}}}), flush=True)
        print(json.dumps({"method":"turn/completed","params":{"threadId":"codex-thread-lineage","turn":{"id":"turn-lineage","status":"completed"}}}), flush=True)
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

    Command::cargo_bin("atelier")
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
            "Lineage task",
        ])
        .assert()
        .success();

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "sessions",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("codex-thread-lineage"))
        .stdout(predicate::str::contains("fake-session-lineage.jsonl"));
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

    (temp, project, thread_id.trim().to_string())
}

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH is set");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
