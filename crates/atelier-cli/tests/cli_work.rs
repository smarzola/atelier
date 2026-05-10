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
fn work_dry_run_writes_job_and_prints_codex_invocation() {
    let (_temp, project, thread_id) = initialized_project();

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "--dry-run",
            "Summarize the project",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("codex exec")
                .and(predicate::str::contains("--cd"))
                .and(predicate::str::contains(
                    project.to_str().expect("utf8 path"),
                ))
                .and(predicate::str::contains("<atelier-context>"))
                .and(predicate::str::contains("Current person: alice"))
                .and(predicate::str::contains("Summarize the project")),
        );

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    let job_dir = &job_dirs[0];
    assert!(job_dir.join("request.md").is_file());
    assert!(job_dir.join("context.md").is_file());
    assert!(job_dir.join("status.json").is_file());
}

#[test]
fn work_executes_fake_codex_and_records_result() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = project.join("codex-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho 'FAKE_CODEX_RESULT'\n",
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
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "Summarize the project",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAKE_CODEX_RESULT"))
        .stdout(predicate::str::contains("Status: succeeded"));

    let argv = std::fs::read_to_string(&recorder).expect("read codex argv");
    assert!(argv.contains("exec\n"));
    assert!(argv.contains("--cd\n"));
    assert!(argv.contains(&format!("{}\n", project.display())));
    assert!(argv.contains("<atelier-context>"));
    assert!(argv.contains("Current person: alice"));
    assert!(argv.contains("Summarize the project"));

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    let job_dir = &job_dirs[0];
    assert_eq!(
        std::fs::read_to_string(job_dir.join("result.md")).expect("read result"),
        "FAKE_CODEX_RESULT\n"
    );
    assert_eq!(
        std::fs::read_to_string(job_dir.join("stderr.log")).expect("read stderr"),
        ""
    );
    let status = std::fs::read_to_string(job_dir.join("status.json")).expect("read status");
    assert!(status.contains("\"status\": \"succeeded\""));
    assert!(status.contains("\"dry_run\": false"));
}
