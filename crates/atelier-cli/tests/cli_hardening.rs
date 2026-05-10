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
            "Hardening",
            "--porcelain",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let thread_id = String::from_utf8(output).expect("utf8 stdout");

    (temp, project, thread_id.trim().to_string())
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
fn failed_work_records_exit_code_and_invocation_metadata() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    std::fs::write(
        &fake_codex,
        "#!/usr/bin/env sh\necho 'simulated stdout'\necho 'simulated stderr' >&2\nexit 7\n",
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
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "Trigger failure",
        ])
        .assert()
        .failure()
        .code(7)
        .stdout(predicate::str::contains("Status: failed"));

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    let job_dir = &job_dirs[0];
    let status = std::fs::read_to_string(job_dir.join("status.json")).expect("read status");
    assert!(status.contains("\"status\": \"failed\""));
    assert!(status.contains("\"exit_code\": 7"));
    assert!(status.contains("\"codex_binary\": \"codex\""));
    assert!(status.contains("\"invocation\":"));
    assert_eq!(
        std::fs::read_to_string(job_dir.join("result.md")).expect("read stdout"),
        "simulated stdout\n"
    );
    assert_eq!(
        std::fs::read_to_string(job_dir.join("stderr.log")).expect("read stderr"),
        "simulated stderr\n"
    );
}
