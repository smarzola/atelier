use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn managed_work_keeps_worker_alive_and_response_completes_turn() {
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
        print(json.dumps({"id": message["id"], "result": {"thread": {"id": "codex-thread-example", "path": "/tmp/fake-session.jsonl"}, "model": "codex-default", "modelProvider": "fake", "cwd": message["params"]["cwd"], "instructionSources": [], "approvalPolicy": "on-request", "approvalsReviewer": "user", "sandbox": {"type": "workspaceWrite"}}}), flush=True)
    elif method == "turn/start":
        print(json.dumps({"id": message["id"], "result": {"turn": {"id": "turn-example", "status": "inProgress"}}}), flush=True)
        print(json.dumps({"method": "item/commandExecution/requestApproval", "id": 7, "params": {"threadId": "codex-thread-example", "turnId": "turn-example", "itemId": "call-example", "reason": "Need test approval", "command": "cargo test", "availableDecisions": ["accept", "decline", "cancel"]}}), flush=True)
        response = json.loads(sys.stdin.readline())
        print(json.dumps({"method": "serverRequest/resolved", "params": {"threadId": "codex-thread-example", "requestId": response["id"]}}), flush=True)
        print(json.dumps({"method": "item/completed", "params": {"item": {"type": "agentMessage", "id": "msg-example", "text": "done"}, "threadId": "codex-thread-example", "turnId": "turn-example"}}), flush=True)
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

    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .env("PATH", prepend_to_path(&fake_bin))
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);

    let assert = Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "--managed",
            "--daemon-url",
            &format!("http://127.0.0.1:{port}"),
            "--idle-timeout-seconds",
            "10",
            "Do the managed thing",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout");
    let job_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Job directory: "))
        .expect("job directory");
    assert!(std::path::Path::new(job_dir).join("worker.json").is_file());
    wait_for_prompt(std::path::Path::new(job_dir), "prompt-7");

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
        .success();

    let status_path = std::path::Path::new(job_dir).join("status.json");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status: Value =
            serde_json::from_str(&std::fs::read_to_string(&status_path).expect("read status"))
                .expect("status json");
        if status["status"] == "succeeded" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "worker did not finish after response: {status}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    let result = std::fs::read_to_string(std::path::Path::new(job_dir).join("result.md"))
        .expect("read result");
    assert!(result.contains("done"));

    let _ = daemon.kill();
}

fn wait_for_prompt(job_dir: &std::path::Path, prompt_id: &str) {
    let prompt_path = job_dir.join("prompts").join(format!("{prompt_id}.json"));
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if prompt_path.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("prompt did not appear: {prompt_id}");
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

fn daemon_command(temp: &tempfile::TempDir, port: u16) -> Command {
    let mut command = Command::cargo_bin("atelier").expect("atelier");
    command
        .env("HOME", temp.path())
        .arg("daemon")
        .arg("run")
        .arg("--listen")
        .arg(format!("127.0.0.1:{port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    listener.local_addr().expect("local addr").port()
}

fn wait_for_health(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(request.as_bytes()).expect("write health");
            let mut response = String::new();
            stream.read_to_string(&mut response).expect("read health");
            if response.contains("\"status\":\"ok\"") {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("daemon did not start");
}

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH is set");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
