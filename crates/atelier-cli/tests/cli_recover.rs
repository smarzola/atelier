use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn jobs_recover_restarts_idle_job_from_saved_request() {
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
        print(json.dumps({"method":"item/completed","params":{"item":{"type":"agentMessage","id":"msg","text":"recovered"},"threadId":"codex-thread","turnId":"turn"}}), flush=True)
        print(json.dumps({"method":"turn/completed","params":{"threadId":"codex-thread","turn":{"id":"turn","status":"completed"}}}), flush=True)
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

    let job_dir = project.join(".atelier/jobs/job-idle");
    std::fs::create_dir_all(&job_dir).expect("create job dir");
    std::fs::write(job_dir.join("request.md"), "Recover this job").expect("write request");
    std::fs::write(job_dir.join("context.md"), "Recover this job").expect("write context");
    std::fs::write(
        job_dir.join("status.json"),
        r#"{"id":"job-idle","status":"idle-timeout","thread_id":"thread-example","person":"alice","dry_run":false,"codex_binary":"codex","invocation":["app-server"]}"#,
    )
    .expect("write status");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .env("PATH", prepend_to_path(&fake_bin))
        .args([
            "jobs",
            "recover",
            project.to_str().expect("utf8 path"),
            "job-idle",
            "--idle-timeout-seconds",
            "10",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Recovered job: job-idle"));

    let status = std::fs::read_to_string(job_dir.join("status.json")).expect("read status");
    assert!(status.contains("\"status\": \"succeeded\""));
    let result = std::fs::read_to_string(job_dir.join("result.md")).expect("read result");
    assert!(result.contains("recovered"));
}

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH is set");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}
