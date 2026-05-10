use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn daemon_work_endpoint_starts_work() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("fake bin");
    write_fake_codex(&fake_bin.join("codex"));

    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .env("PATH", prepend_to_path(&fake_bin))
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);

    let response = post_json(
        port,
        "/work",
        &format!(
            r#"{{"project":"example-project","thread":"{}","person":"alice","text":"Run daemon task"}}"#,
            thread_id
        ),
    );
    assert_eq!(response["status"], "started");
    assert_eq!(response["project"], "example-project");
    assert_eq!(response["thread"], thread_id);
    assert_eq!(response["person"], "alice");
    wait_for_job_success(&project, response["job_id"].as_str().expect("job id"));

    let audit_event = latest_audit_event(&temp, "work_started");
    assert_eq!(audit_event["action"], "work_started");
    assert_eq!(audit_event["project"], "example-project");
    assert_eq!(audit_event["person"], "alice");
    assert_eq!(audit_event["result"], "started");

    let _ = daemon.kill();
}

#[test]
fn daemon_run_hosts_gateway_health_endpoint() {
    let temp = tempfile::tempdir().expect("tempdir");
    let port = free_port();
    let mut daemon = daemon_command(&temp, port).spawn().expect("spawn daemon");
    wait_for_health(port);

    let health = get_json(port, "/health");
    assert_eq!(health["status"], "ok");

    let _ = daemon.kill();
}

#[test]
fn daemon_run_supervises_workers_by_default() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    write_running_job_with_dead_worker(&project, "job-dead-worker");

    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .arg("--supervision-interval-millis")
        .arg("50")
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);
    wait_for_job_status(&project, "job-dead-worker", "worker-lost");

    let _ = daemon.kill();
}

fn wait_for_job_success(project: &std::path::Path, job_id: &str) {
    wait_for_job_status(project, job_id, "succeeded")
}

fn latest_audit_event(temp: &tempfile::TempDir, action: &str) -> Value {
    let audit_path = temp.path().join(".atelier/gateway/audit.jsonl");
    let content = std::fs::read_to_string(audit_path).expect("read audit log");
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .rev()
        .find(|event| event["action"] == action)
        .unwrap_or_else(|| panic!("missing audit event for action {action}"))
}

fn init_and_register(temp: &tempfile::TempDir, project: &std::path::Path) {
    Command::cargo_bin("atelier")
        .expect("atelier")
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
        .expect("atelier")
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

fn create_thread(temp: &tempfile::TempDir, project: &std::path::Path) -> String {
    let output = Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .args([
            "thread",
            "new",
            project.to_str().expect("utf8 path"),
            "Daemon thread",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("utf8").trim().to_string()
}

fn write_running_job_with_dead_worker(project: &std::path::Path, job_id: &str) {
    let job_dir = project.join(".atelier/jobs").join(job_id);
    std::fs::create_dir_all(&job_dir).expect("job dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": job_id,
            "status": "running",
            "thread_id":"thread-example",
            "person":"alice",
            "dry_run":false,
            "codex_binary":"codex",
            "invocation":["app-server"]
        }))
        .expect("status json"),
    )
    .expect("status");
    std::fs::write(
        job_dir.join("worker.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "pid": 99999999_u64,
            "idle_timeout_seconds": 300
        }))
        .expect("worker json"),
    )
    .expect("worker file");
}

fn wait_for_job_status(project: &std::path::Path, job_id: &str, expected: &str) {
    let status_path = project
        .join(".atelier/jobs")
        .join(job_id)
        .join("status.json");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status: Value =
            serde_json::from_str(&std::fs::read_to_string(&status_path).expect("read job status"))
                .expect("status json");
        if status["status"] == expected {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "job did not reach {expected}: {status}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
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
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("daemon did not start");
}

fn get_json(port: u16, path: &str) -> Value {
    request_json(port, "GET", path, "")
}

fn post_json(port: u16, path: &str, body: &str) -> Value {
    request_json(port, "POST", path, body)
}

fn request_json(port: u16, method: &str, path: &str, body: &str) -> Value {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect daemon");
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    let body = response.split("\r\n\r\n").nth(1).expect("response body");
    serde_json::from_str(body).expect("json body")
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
        print(json.dumps({"method":"item/completed","params":{"item":{"type":"agentMessage","id":"msg","text":"daemon done"},"threadId":"codex-thread","turnId":"turn"}}), flush=True)
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

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
