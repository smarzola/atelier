use assert_cmd::Command;
use predicates::prelude::*;

fn initialized_project() -> (tempfile::TempDir, std::path::PathBuf, String) {
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

fn prepend_to_path(dir: &std::path::Path) -> std::ffi::OsString {
    let original_path = std::env::var_os("PATH").expect("PATH is set");
    std::env::join_paths(
        std::iter::once(dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH")
}

#[test]
fn continue_last_invokes_codex_exec_resume_and_records_result() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = project.join("codex-resume-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho 'FAKE_RESUME_RESULT'\n",
            recorder.display()
        ),
    )
    .expect("write fake codex");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake_codex, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("PATH", prepend_to_path(&fake_bin))
        .args([
            "continue",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "--last",
            "Continue the previous work",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAKE_RESUME_RESULT"))
        .stdout(predicate::str::contains("Status: succeeded"));

    let argv = std::fs::read_to_string(&recorder).expect("read codex argv");
    assert!(argv.contains("exec\n"));
    assert!(argv.contains("resume\n"));
    assert!(argv.contains("--last\n"));
    assert!(argv.contains("Continue the previous work"));

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    assert_eq!(
        std::fs::read_to_string(job_dirs[0].join("result.md")).expect("read result"),
        "FAKE_RESUME_RESULT\n"
    );
}

#[test]
fn sessions_list_prints_thread_session_lineage() {
    let (_temp, project, thread_id) = initialized_project();
    let lineage = project
        .join(".atelier/threads")
        .join(&thread_id)
        .join("codex-sessions.jsonl");
    std::fs::write(&lineage, "{\"session_id\":\"session-example\"}\n").expect("write lineage");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "sessions",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("session-example"));
}
