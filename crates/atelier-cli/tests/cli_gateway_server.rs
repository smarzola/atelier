use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn gateway_health_status_jobs_prompts_and_respond_endpoints_work() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    write_waiting_prompt_job(&project, "job-gateway", "prompt-gateway");

    let port = free_port();
    let mut server = spawn_gateway(&temp, port);
    wait_for_health(port);

    let health = get_json(port, "/health");
    assert_eq!(health["status"], "ok");

    let status = get_json(port, "/status");
    assert_eq!(status["projects"], 1);
    assert_eq!(status["waiting_prompts"], 1);

    let jobs = get_json(port, "/jobs");
    assert_eq!(jobs["jobs"][0]["id"], "job-gateway");

    let prompts = get_json(port, "/prompts");
    assert_eq!(prompts["prompts"][0]["id"], "prompt-gateway");

    let response = post_json(
        port,
        "/prompts/respond",
        r#"{"project":"example-project","prompt_id":"prompt-gateway","decision":"answer","text":"gateway answer"}"#,
    );
    assert_eq!(response["status"], "recorded");
    let response_file = project.join(".atelier/jobs/job-gateway/responses/prompt-gateway.json");
    let response_text = std::fs::read_to_string(response_file).expect("read response file");
    assert!(response_text.contains("gateway answer"));

    let _ = server.kill();
}

#[test]
fn gateway_message_event_starts_managed_work() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("fake bin");
    write_fake_codex(&fake_bin.join("codex"));

    let port = free_port();
    let mut server = spawn_gateway_with_path(&temp, port, &fake_bin);
    wait_for_health(port);

    let response = post_json(
        port,
        "/events/message",
        &format!(
            r#"{{"gateway":"example-gateway","project":"example-project","thread":"{}","person":"alice","text":"Run gateway task"}}"#,
            thread_id
        ),
    );
    assert_eq!(response["status"], "started");
    wait_for_job_success(&project, response["job_id"].as_str().expect("job id"));

    let _ = server.kill();
}

#[test]
fn gateway_message_event_resolves_bound_thread_and_person() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .args([
            "gateway",
            "bind",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--gateway",
            "example-gateway",
            "--external-thread",
            "external-thread",
        ])
        .assert()
        .success();
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .args([
            "gateway",
            "bind-person",
            "--gateway",
            "example-gateway",
            "--external-user",
            "external-user",
            "--person",
            "alice",
        ])
        .assert()
        .success();

    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("fake bin");
    write_fake_codex(&fake_bin.join("codex"));

    let port = free_port();
    let mut server = spawn_gateway_with_path(&temp, port, &fake_bin);
    wait_for_health(port);

    let response = post_json(
        port,
        "/events/message",
        r#"{"gateway":"example-gateway","external_thread":"external-thread","external_user":"external-user","text":"Run resolved gateway task"}"#,
    );
    assert_eq!(response["status"], "started");
    assert_eq!(response["project"], "example-project");
    assert_eq!(response["thread"], thread_id);
    assert_eq!(response["person"], "alice");
    wait_for_job_success(&project, response["job_id"].as_str().expect("job id"));

    let _ = server.kill();
}

#[test]
fn telegram_adapter_message_update_starts_managed_work_through_generic_gateway() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
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
    write_fake_codex(&fake_bin.join("codex"));

    let port = free_port();
    let mut server = spawn_gateway_with_path(&temp, port, &fake_bin);
    wait_for_health(port);

    let response = post_json(
        port,
        "/adapters/telegram/update",
        r#"{"update_id":1,"message":{"message_id":10,"message_thread_id":77,"chat":{"id":1000,"type":"supergroup"},"from":{"id":2000,"is_bot":false,"first_name":"Example"},"text":"Run Telegram task"}}"#,
    );
    assert_eq!(response["status"], "started");
    assert_eq!(response["project"], "example-project");
    assert_eq!(response["thread"], thread_id);
    assert_eq!(response["person"], "alice");
    wait_for_job_success(&project, response["job_id"].as_str().expect("job id"));

    let _ = server.kill();
}

#[test]
fn gateway_rejects_non_loopback_listen_without_explicit_opt_in() {
    let temp = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .args(["gateway", "serve", "--listen", "0.0.0.0:0"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "refusing to listen on non-loopback address",
        ));
}

#[test]
fn gateway_requires_bearer_token_when_auth_token_env_is_set() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);

    let port = free_port();
    let mut server =
        spawn_gateway_with_auth(&temp, port, "ATELIER_TEST_GATEWAY_TOKEN", "secret-token");
    wait_for_health(port);

    let unauthenticated = get_status_and_json(port, "/status", None);
    assert_eq!(unauthenticated.0, 401);
    assert_eq!(unauthenticated.1["error"], "unauthorized");

    let authenticated = get_status_and_json(port, "/status", Some("secret-token"));
    assert_eq!(authenticated.0, 200);
    assert_eq!(authenticated.1["projects"], 1);

    let _ = server.kill();
}

#[test]
fn gateway_fails_fast_when_auth_token_env_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin("atelier")
        .expect("atelier")
        .env("HOME", temp.path())
        .env_remove("ATELIER_MISSING_GATEWAY_TOKEN")
        .args([
            "gateway",
            "serve",
            "--listen",
            "127.0.0.1:0",
            "--auth-token-env",
            "ATELIER_MISSING_GATEWAY_TOKEN",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("auth token env var is not set"));
}

fn wait_for_job_success(project: &std::path::Path, job_id: &str) {
    let status_path = project
        .join(".atelier/jobs")
        .join(job_id)
        .join("status.json");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status: Value =
            serde_json::from_str(&std::fs::read_to_string(&status_path).expect("read job status"))
                .expect("status json");
        if status["status"] == "succeeded" {
            break;
        }
        assert!(Instant::now() < deadline, "job did not complete: {status}");
        std::thread::sleep(Duration::from_millis(100));
    }
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
            "Gateway thread",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("utf8").trim().to_string()
}

fn write_waiting_prompt_job(project: &std::path::Path, job_id: &str, prompt_id: &str) {
    let job_dir = project.join(".atelier/jobs").join(job_id);
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("prompts dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": job_id,
            "status":"waiting-for-prompt",
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
        prompts_dir.join(format!("{prompt_id}.json")),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": prompt_id,
            "codex_request_id":"1",
            "method":"item/tool/requestUserInput",
            "codex_thread_id":"codex-thread-example",
            "codex_turn_id":"turn-example",
            "codex_item_id":"item-example",
            "status":"Pending",
            "summary":"Answer gateway prompt",
            "available_decisions":["answer","cancel"],
            "params":{}
        }))
        .expect("prompt json"),
    )
    .expect("prompt");
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
        print(json.dumps({"method":"item/completed","params":{"item":{"type":"agentMessage","id":"msg","text":"gateway done"},"threadId":"codex-thread","turnId":"turn"}}), flush=True)
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

fn spawn_gateway(temp: &tempfile::TempDir, port: u16) -> std::process::Child {
    spawn_gateway_with_path(temp, port, std::path::Path::new(""))
}

fn spawn_gateway_with_auth(
    temp: &tempfile::TempDir,
    port: u16,
    token_env: &str,
    token: &str,
) -> std::process::Child {
    let mut command = gateway_command(temp, port);
    command
        .env(token_env, token)
        .arg("--auth-token-env")
        .arg(token_env)
        .spawn()
        .expect("spawn gateway")
}

fn spawn_gateway_with_path(
    temp: &tempfile::TempDir,
    port: u16,
    fake_bin: &std::path::Path,
) -> std::process::Child {
    let mut command = gateway_command(temp, port);
    if !fake_bin.as_os_str().is_empty() {
        command.env("PATH", prepend_to_path(fake_bin));
    }
    command.spawn().expect("spawn gateway")
}

fn gateway_command(temp: &tempfile::TempDir, port: u16) -> Command {
    let mut command = Command::cargo_bin("atelier").expect("atelier");
    command
        .env("HOME", temp.path())
        .arg("gateway")
        .arg("serve")
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
    panic!("gateway did not start");
}

fn get_json(port: u16, path: &str) -> Value {
    get_status_and_json(port, path, None).1
}

fn post_json(port: u16, path: &str, body: &str) -> Value {
    request_status_and_json(port, "POST", path, body, None).1
}

fn get_status_and_json(port: u16, path: &str, token: Option<&str>) -> (u16, Value) {
    request_status_and_json(port, "GET", path, "", token)
}

fn request_status_and_json(
    port: u16,
    method: &str,
    path: &str,
    body: &str,
    token: Option<&str>,
) -> (u16, Value) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect gateway");
    let auth_header = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n{auth_header}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
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

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
