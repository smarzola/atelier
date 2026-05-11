use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::mpsc;
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
fn daemon_events_endpoint_returns_thread_events_after_sequence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    atelier_core::thread_events::append_thread_event(
        &project,
        &thread_id,
        Some("job-example"),
        "final_result",
        serde_json::json!({"text":"Example result"}),
    )
    .expect("append first event");
    atelier_core::thread_events::append_thread_event(
        &project,
        &thread_id,
        Some("job-example"),
        "job_succeeded",
        serde_json::json!({"status":"succeeded"}),
    )
    .expect("append second event");

    let port = free_port();
    let mut daemon = daemon_command(&temp, port).spawn().expect("spawn daemon");
    wait_for_health(port);

    let events = get_json(
        port,
        &format!("/events?project=example-project&thread={thread_id}&after=1"),
    );
    assert_eq!(events["last_sequence"], 2);
    assert_eq!(events["events"].as_array().expect("events").len(), 1);
    assert_eq!(events["events"][0]["kind"], "job_succeeded");

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

#[test]
fn daemon_message_endpoint_starts_after_dead_worker_without_waiting_for_supervisor() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    write_running_job_with_dead_worker_for_thread(&project, "job-dead-worker", &thread_id);

    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("fake bin");
    write_fake_codex(&fake_bin.join("codex"));

    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .env("PATH", prepend_to_path(&fake_bin))
        .arg("--supervision-interval-millis")
        .arg("60000")
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);

    let response = post_json(
        port,
        "/events/message",
        &format!(
            r#"{{"gateway":"example-gateway","project":"example-project","thread":"{}","person":"alice","text":"Run after stale worker"}}"#,
            thread_id
        ),
    );
    assert_eq!(
        response["status"], "started",
        "dead worker should not keep owning the thread/project writer slot: {response}"
    );
    wait_for_job_status(&project, "job-dead-worker", "worker-lost");
    wait_for_job_success(&project, response["job_id"].as_str().expect("job id"));

    let jobs = get_json(port, "/jobs");
    let dead_worker = jobs["jobs"]
        .as_array()
        .expect("jobs")
        .iter()
        .find(|job| job["id"] == "job-dead-worker")
        .expect("dead worker job listed");
    assert_eq!(dead_worker["status"], "worker-lost");

    let _ = daemon.kill();
}

#[test]
fn daemon_run_sets_telegram_webhook_and_enforces_update_secret() {
    let temp = tempfile::tempdir().expect("tempdir");
    let telegram_api = FakeTelegramApi::start(1);
    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .env("ATELIER_TELEGRAM_BOT_TOKEN", "example-token")
        .env("ATELIER_TELEGRAM_API_BASE", telegram_api.base_url())
        .env(
            "ATELIER_TELEGRAM_WEBHOOK_URL",
            "https://example.invalid/atelier/telegram",
        )
        .env("ATELIER_TELEGRAM_WEBHOOK_SECRET", "example-secret")
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);

    let setup = post_json(port, "/adapters/telegram/webhook/setup", "{}");
    assert_eq!(setup["status"], "configured");
    assert_eq!(setup["result"]["ok"], true);

    let request = telegram_api.next_request();
    assert_eq!(request.path, "/botexample-token/setWebhook");
    let body: Value = serde_json::from_str(&request.body).expect("webhook body");
    assert_eq!(body["url"], "https://example.invalid/atelier/telegram");
    assert_eq!(body["secret_token"], "example-secret");

    let rejected = request_status_and_json(
        port,
        "POST",
        "/adapters/telegram/update",
        r#"{"update_id":1}"#,
        &[("X-Telegram-Bot-Api-Secret-Token", "wrong-secret")],
    );
    assert_eq!(rejected.0, 401);
    assert_eq!(rejected.1["error"], "unauthorized");

    let accepted = request_status_and_json(
        port,
        "POST",
        "/adapters/telegram/update",
        r#"{"update_id":1}"#,
        &[("X-Telegram-Bot-Api-Secret-Token", "example-secret")],
    );
    assert_ne!(accepted.0, 401);

    let _ = daemon.kill();
}

#[test]
fn daemon_run_sends_telegram_messages_through_bot_api() {
    let temp = tempfile::tempdir().expect("tempdir");
    let telegram_api = FakeTelegramApi::start(1);
    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .env("ATELIER_TELEGRAM_BOT_TOKEN", "example-token")
        .env("ATELIER_TELEGRAM_API_BASE", telegram_api.base_url())
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);

    let response = post_json(
        port,
        "/adapters/telegram/send-message",
        r#"{"chat_id":"1000","message_thread_id":"77","text":"Example notification"}"#,
    );
    assert_eq!(response["status"], "sent");
    assert_eq!(response["result"]["ok"], true);

    let request = telegram_api.next_request();
    assert_eq!(request.path, "/botexample-token/sendMessage");
    let body: Value = serde_json::from_str(&request.body).expect("sendMessage body");
    assert_eq!(body["chat_id"], "1000");
    assert_eq!(body["message_thread_id"], "77");
    assert_eq!(body["text"], "Example notification");

    let _ = daemon.kill();
}

#[test]
fn daemon_run_acknowledges_telegram_update_job_start() {
    telegram_update_job_start_with_fake_codex("daemon done", 2, |bodies, job_id| {
        assert!(
            bodies
                .iter()
                .any(|body| body["text"].as_str().expect("ack text").contains(job_id)),
            "one Telegram message should include job id: {bodies:?}"
        );
        let final_body = bodies
            .iter()
            .find(|body| body["text"] == "daemon done")
            .expect("final result message");
        assert_eq!(final_body["chat_id"], "1000");
        assert_eq!(final_body["message_thread_id"], "77");
    });
}

#[test]
fn daemon_run_coalesces_telegram_progress_before_final_result() {
    telegram_update_job_start_with_fake_codex(
        "progress: drafting|daemon done|daemon done",
        2,
        |bodies, job_id| {
            assert!(
                bodies
                    .iter()
                    .any(|body| body["text"].as_str().expect("ack text").contains(job_id)),
                "one Telegram message should include job id: {bodies:?}"
            );
            assert!(
                bodies
                    .iter()
                    .all(|body| body["text"] != "progress: drafting"),
                "stale progress snapshot should be coalesced away: {bodies:?}"
            );
            assert!(
                bodies
                    .iter()
                    .all(|body| body["text"] != "progress: final draft"),
                "progress snapshot that matches the final result should be coalesced away: {bodies:?}"
            );
            assert!(
                bodies.iter().any(|body| body["text"] == "daemon done"),
                "final result should still be delivered: {bodies:?}"
            );
        },
    );
}

fn telegram_update_job_start_with_fake_codex(
    fake_messages: &str,
    expected_telegram_requests: usize,
    assert_bodies: impl FnOnce(&[Value], &str),
) {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .args([
            "gateway",
            "bind",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--gateway",
            "telegram",
            "--external-thread",
            "chat:1000:topic:77",
        ])
        .assert()
        .success();
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
        .args([
            "gateway",
            "bind-person",
            "--gateway",
            "telegram",
            "--external-user",
            "2000",
            "--person",
            "alice",
        ])
        .assert()
        .success();
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("fake bin");
    write_fake_codex_with_messages(&fake_bin.join("codex"), fake_messages);
    let telegram_api = FakeTelegramApi::start(expected_telegram_requests);
    let port = free_port();
    let mut daemon = daemon_command(&temp, port)
        .env("PATH", prepend_to_path(&fake_bin))
        .env("ATELIER_TELEGRAM_BOT_TOKEN", "example-token")
        .env("ATELIER_TELEGRAM_API_BASE", telegram_api.base_url())
        .spawn()
        .expect("spawn daemon");
    wait_for_health(port);

    let response = post_json(
        port,
        "/adapters/telegram/update",
        r#"{"update_id":1,"message":{"message_id":10,"message_thread_id":77,"chat":{"id":1000,"type":"supergroup"},"from":{"id":2000,"is_bot":false,"first_name":"Example"},"text":"Run Telegram task"}}"#,
    );
    assert_eq!(response["status"], "started");
    let job_id = response["job_id"].as_str().expect("job id");
    wait_for_job_success(&project, job_id);

    let mut bodies = Vec::new();
    for index in 0..expected_telegram_requests {
        let request = telegram_api.next_request();
        assert_eq!(request.path, "/botexample-token/sendMessage");
        bodies.push(
            serde_json::from_str(&request.body)
                .unwrap_or_else(|error| panic!("telegram body {index} should be JSON: {error}")),
        );
    }
    assert_bodies(&bodies, job_id);

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
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env("ATELIER_HOME", temp.path().join(".atelier"))
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
        .env("ATELIER_HOME", temp.path().join(".atelier"))
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
    write_running_job_with_dead_worker_for_thread(project, job_id, "thread-example");
}

fn write_running_job_with_dead_worker_for_thread(
    project: &std::path::Path,
    job_id: &str,
    thread_id: &str,
) {
    let job_dir = project.join(".atelier/jobs").join(job_id);
    std::fs::create_dir_all(&job_dir).expect("job dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": job_id,
            "status": "running",
            "thread_id":thread_id,
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
        .env("ATELIER_HOME", temp.path().join(".atelier"))
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
    request_status_and_json(port, method, path, body, &[]).1
}

fn request_status_and_json(
    port: u16,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> (u16, Value) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect daemon");
    let extra_headers = headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect::<String>();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n{extra_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .expect("status code");
    let body = response.split("\r\n\r\n").nth(1).expect("response body");
    (status, serde_json::from_str(body).expect("json body"))
}

fn write_fake_codex(path: &std::path::Path) {
    write_fake_codex_with_messages(path, "daemon done");
}

fn write_fake_codex_with_messages(path: &std::path::Path, messages: &str) {
    let script = format!(
        r#"#!/usr/bin/env python3
import json, sys
messages = {messages:?}.split('|')
for line in sys.stdin:
    message=json.loads(line)
    if message.get("method") == "initialize":
        print(json.dumps({{"id":message["id"],"result":{{"userAgent":"fake","codexHome":"/tmp/fake","platformFamily":"unix","platformOs":"linux"}}}}), flush=True)
    elif message.get("method") == "initialized":
        continue
    elif message.get("method") == "thread/start":
        print(json.dumps({{"id":message["id"],"result":{{"thread":{{"id":"codex-thread","path":"/tmp/session.jsonl"}},"model":"default","modelProvider":"fake","cwd":message["params"]["cwd"],"instructionSources":[],"approvalPolicy":"on-request","approvalsReviewer":"user","sandbox":{{"type":"workspaceWrite"}}}}}}), flush=True)
    elif message.get("method") == "turn/start":
        print(json.dumps({{"id":message["id"],"result":{{"turn":{{"id":"turn","status":"inProgress"}}}}}}), flush=True)
        for index, text in enumerate(messages):
            print(json.dumps({{"method":"item/completed","params":{{"item":{{"type":"agentMessage","id":f"msg-{{index}}","text":text}},"threadId":"codex-thread","turnId":"turn"}}}}), flush=True)
        print(json.dumps({{"method":"turn/completed","params":{{"threadId":"codex-thread","turn":{{"id":"turn","status":"completed"}}}}}}), flush=True)
        break
"#
    );
    std::fs::write(path, script).expect("fake codex");
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

struct FakeTelegramApi {
    port: u16,
    receiver: mpsc::Receiver<CapturedRequest>,
}

struct CapturedRequest {
    path: String,
    body: String,
}

impl FakeTelegramApi {
    fn start(expected_requests: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake telegram api");
        let port = listener.local_addr().expect("local addr").port();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().expect("accept fake telegram request");
                let request = read_raw_http_request(&mut stream);
                sender.send(request).expect("send captured request");
                let body = r#"{"ok":true,"result":{"message_id":123}}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write fake response");
            }
        });
        Self { port, receiver }
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    fn next_request(&self) -> CapturedRequest {
        self.receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("captured telegram request")
    }
}

fn read_raw_http_request(stream: &mut TcpStream) -> CapturedRequest {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    loop {
        let bytes = stream.read(&mut temp).expect("read fake request");
        if bytes == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..bytes]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    let header_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .expect("header end");
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let request_line = headers.lines().next().expect("request line");
    let path = request_line
        .split_whitespace()
        .nth(1)
        .expect("request path")
        .to_string();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    while buffer.len() < header_end + content_length {
        let bytes = stream.read(&mut temp).expect("read fake body");
        if bytes == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..bytes]);
    }
    CapturedRequest {
        path,
        body: String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).to_string(),
    }
}
