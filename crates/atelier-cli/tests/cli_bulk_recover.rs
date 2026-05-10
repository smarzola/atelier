use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn jobs_recover_all_idle_recovers_matching_jobs_in_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
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

    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    std::fs::write(
        &fake_codex,
        r#"#!/usr/bin/env python3
import json
import sys
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
        print(json.dumps({"method":"item/completed","params":{"item":{"type":"agentMessage","id":"msg","text":"bulk recovered"},"threadId":"codex-thread","turnId":"turn"}}), flush=True)
        print(json.dumps({"method":"turn/completed","params":{"threadId":"codex-thread","turn":{"id":"turn","status":"completed"}}}), flush=True)
        break
"#,
    )
    .expect("write fake codex");
    chmod_executable(&fake_codex);

    write_recoverable_job(&project, "job-idle-one", "idle-timeout");
    write_recoverable_job(&project, "job-idle-two", "idle-timeout");
    write_recoverable_job(&project, "job-worker-lost", "worker-lost");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .env("PATH", prepend_to_path(&fake_bin))
        .args([
            "jobs",
            "recover",
            project.to_str().expect("utf8 path"),
            "--all-idle",
            "--idle-timeout-seconds",
            "10",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Recovered job: job-idle-one"))
        .stdout(predicate::str::contains("Recovered job: job-idle-two"))
        .stdout(predicate::str::contains("Recovered 2 jobs"));

    let worker_lost_status = std::fs::read_to_string(
        project
            .join(".atelier/jobs")
            .join("job-worker-lost")
            .join("status.json"),
    )
    .expect("read worker-lost status");
    assert!(worker_lost_status.contains("worker-lost"));
}

fn write_recoverable_job(project: &std::path::Path, job_id: &str, status: &str) {
    let job_dir = project.join(".atelier/jobs").join(job_id);
    std::fs::create_dir_all(&job_dir).expect("create job dir");
    std::fs::write(job_dir.join("request.md"), "Recover this job").expect("write request");
    std::fs::write(job_dir.join("context.md"), "Recover this job").expect("write context");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": job_id,
            "status": status,
            "thread_id":"thread-example",
            "person":"alice",
            "dry_run":false,
            "codex_binary":"codex",
            "invocation":["app-server"]
        }))
        .expect("serialize status"),
    )
    .expect("write status");
}

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH is set");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}

fn chmod_executable(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod executable");
    }
}
