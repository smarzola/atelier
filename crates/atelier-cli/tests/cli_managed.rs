use assert_cmd::Command;

#[test]
fn work_requires_running_daemon() {
    let (temp, project, thread_id) = initialized_project();
    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    std::fs::write(&fake_codex, "#!/usr/bin/env python3\n").expect("write fake codex");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake_codex, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }

    Command::cargo_bin("atelier")
        .expect("atelier binary")
        .env("HOME", temp.path())
        .env("PATH", prepend_to_path(&fake_bin))
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "Do the managed thing",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "work requires a running Atelier daemon",
        ));

    let jobs_dir = project.join(".atelier/jobs");
    let job_count = if jobs_dir.exists() {
        std::fs::read_dir(jobs_dir).expect("read jobs dir").count()
    } else {
        0
    };
    assert_eq!(job_count, 0);
}

fn initialized_project() -> (tempfile::TempDir, std::path::PathBuf, String) {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
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

    let output = Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("HOME", temp.path())
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
