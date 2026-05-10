use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn work_accepts_registered_project_alias() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = temp.path().join("codex-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho alias-work-ok\n",
            recorder.display()
        ),
    )
    .expect("write fake codex");
    chmod_executable(&fake_codex);

    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .env("PATH", prepend_to_path(&fake_bin))
        .args([
            "work",
            "example-project",
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "Use alias",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("alias-work-ok"));

    let argv = std::fs::read_to_string(recorder).expect("read argv");
    assert!(argv.contains(project.to_str().expect("utf8 path")));
}

#[test]
fn jobs_prompts_and_sessions_accept_registered_project_aliases() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    init_and_register(&temp, &project);
    let thread_id = create_thread(&temp, &project);
    let job_dir = project.join(".atelier/jobs/job-alias");
    let prompts_dir = job_dir.join("prompts");
    std::fs::create_dir_all(&prompts_dir).expect("create prompts dir");
    std::fs::write(
        job_dir.join("status.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id":"job-alias",
            "status":"waiting-for-prompt",
            "thread_id":thread_id,
            "person":"alice",
            "dry_run":false,
            "codex_binary":"codex",
            "invocation":["app-server"]
        }))
        .expect("serialize status"),
    )
    .expect("write status");
    std::fs::write(
        prompts_dir.join("prompt-1.json"),
        r#"{
  "id": "prompt-1",
  "codex_request_id": "1",
  "method": "item/tool/requestUserInput",
  "codex_thread_id": "codex-thread-example",
  "codex_turn_id": "turn-example",
  "codex_item_id": "item-example",
  "status": "Pending",
  "summary": "Answer tool user-input prompt",
  "available_decisions": ["answer", "cancel"],
  "params": {}
}
"#,
    )
    .expect("write prompt");
    std::fs::write(
        project
            .join(".atelier/threads")
            .join(&thread_id)
            .join("codex-sessions.jsonl"),
        "{\"codex_thread_id\":\"codex-thread-example\"}\n",
    )
    .expect("write sessions");

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["jobs", "list", "example-project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("job-alias"));

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["prompts", "list", "example-project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("prompt-1"));

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args(["sessions", "example-project", "--thread", &thread_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("codex-thread-example"));
}

fn init_and_register(temp: &tempfile::TempDir, project: &std::path::Path) {
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
    Command::cargo_bin("atelier")
        .expect("atelier binary")
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
        .expect("atelier binary")
        .env("HOME", temp.path())
        .args([
            "thread",
            "new",
            project.to_str().expect("utf8 path"),
            "Alias thread",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output)
        .expect("utf8 stdout")
        .trim()
        .to_string()
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
