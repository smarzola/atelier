use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect daemon");
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    let body = response.split("\r\n\r\n").nth(1).expect("response body");
    serde_json::from_str(body).expect("json body")
}
