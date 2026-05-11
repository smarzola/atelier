use assert_cmd::Command;
use predicates::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

#[test]
fn thread_new_creates_thread_and_threads_list_shows_it() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
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
        .args([
            "thread",
            "new",
            project.to_str().expect("utf8 path"),
            "Release preparation",
            "--porcelain",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("thread-"))
        .get_output()
        .stdout
        .clone();

    let thread_id = String::from_utf8(output).expect("utf8 stdout");
    let thread_id = thread_id.trim();
    let thread_dir = project.join(".atelier/threads").join(thread_id);

    assert!(thread_dir.join("thread.toml").is_file());
    assert!(thread_dir.join("summary.md").is_file());
    assert!(thread_dir.join("gateway-bindings.toml").is_file());
    assert!(thread_dir.join("codex-sessions.jsonl").is_file());
    assert!(thread_dir.join("jobs").is_dir());

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args(["threads", "list", project.to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("Release preparation"));
}

#[test]
fn thread_send_submits_to_daemon_and_thread_follow_reads_items() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_project(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("fake bin");
    write_fake_codex(&fake_bin.join("codex"));

    let port = free_port();
    let mut daemon = std::process::Command::new(assert_cmd::cargo::cargo_bin("atelier"));
    daemon
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .env("PATH", prepend_to_path(&fake_bin))
        .arg("daemon")
        .arg("run")
        .arg("--listen")
        .arg(format!("127.0.0.1:{port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut daemon = daemon.spawn().expect("spawn daemon");
    wait_for_health(port);

    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .args([
            "thread",
            "send",
            "example-project",
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "--daemon-url",
            &format!("http://127.0.0.1:{port}"),
            "Write a result",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: started"))
        .stdout(predicate::str::contains("Job:").not())
        .stdout(predicate::str::contains("Job directory:").not());

    let items_path = project
        .join(".atelier/threads")
        .join(&thread_id)
        .join("items.jsonl");
    wait_for_file_contains(&items_path, "thread send done");

    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .args([
            "thread",
            "follow",
            "example-project",
            "--thread",
            &thread_id,
            "--after",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1\tmessage\tuser\tWrite a result"))
        .stdout(predicate::str::contains(
            "message\tassistant\tthread send done",
        ))
        .stdout(predicate::str::contains("final_result").not());

    let _ = daemon.kill();
}

fn init_project(temp: &tempfile::TempDir, project: &std::path::Path) {
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .args([
            "project",
            "init",
            project.to_str().expect("utf8 path"),
            "--name",
            "example-project",
        ])
        .assert()
        .success();
}

fn create_thread(temp: &tempfile::TempDir, project: &std::path::Path) -> String {
    let output = Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .args([
            "thread",
            "new",
            project.to_str().expect("utf8 path"),
            "Example thread",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("utf8").trim().to_string()
}

fn write_fake_codex(path: &std::path::Path) {
    std::fs::write(
        path,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    message=json.loads(line)
    if message.get("method") == "initialize":
        print(json.dumps({"id":message["id"],"result":{"userAgent":"fake","codexHome":"/tmp/fake","platformFamily":"unix","platformOs":"linux"}}), flush=True)
    elif message.get("method") == "initialized":
        continue
    elif message.get("method") == "thread/start":
        print(json.dumps({"id":message["id"],"result":{"thread":{"id":"codex-thread","path":"/tmp/session.jsonl"},"model":"default","modelProvider":"fake","cwd":message["params"]["cwd"],"instructionSources":[],"approvalPolicy":"on-request","approvalsReviewer":"user","sandbox":{"type":"workspaceWrite"}}}), flush=True)
    elif message.get("method") == "turn/start":
        print(json.dumps({"id":message["id"],"result":{"turn":{"id":"turn","status":"inProgress"}}}), flush=True)
        print(json.dumps({"method":"item/completed","params":{"item":{"type":"agentMessage","id":"msg","text":"thread send done"},"threadId":"codex-thread","turnId":"turn"}}), flush=True)
        print(json.dumps({"method":"turn/completed","params":{"threadId":"codex-thread","turn":{"id":"turn","status":"completed"}}}), flush=True)
        break
"#,
    )
    .expect("fake codex");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    listener.local_addr().expect("local addr").port()
}

fn wait_for_health(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("daemon did not start");
}

fn wait_for_file_contains(path: &std::path::Path, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if std::fs::read_to_string(path)
            .map(|content| content.contains(expected))
            .unwrap_or(false)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("file did not contain {expected}: {}", path.display());
}

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
