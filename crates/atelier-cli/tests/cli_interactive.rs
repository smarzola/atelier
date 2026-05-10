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
            "Interactive dogfood",
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
fn work_interactive_streams_codex_prompts_to_stdout_and_records_job() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = project.join("interactive-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho 'Codex asks: approve shell command?'\necho 'INTERACTIVE_DONE'\n",
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
            "--interactive",
            "Run a task that may ask for approval",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Codex asks: approve shell command?",
        ))
        .stdout(predicate::str::contains("INTERACTIVE_DONE"))
        .stdout(predicate::str::contains("Status: succeeded"));

    let argv = std::fs::read_to_string(&recorder).expect("read codex argv");
    assert!(argv.contains("exec\n"));
    assert!(argv.contains("--cd\n"));
    assert!(argv.contains("Run a task that may ask for approval"));

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    assert!(job_dirs[0].join("result.md").is_file());
    assert_eq!(
        std::fs::read_to_string(job_dirs[0].join("interactive-output.md"))
            .expect("read interactive output note"),
        "Interactive job output was streamed directly to the attached terminal.\n"
    );
}

#[test]
fn work_interactive_does_not_add_model_when_model_is_omitted() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = project.join("interactive-no-model-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho 'NO_MODEL_DONE'\n",
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
            "--interactive",
            "Use Codex default model",
        ])
        .assert()
        .success();

    let argv = std::fs::read_to_string(&recorder).expect("read codex argv");
    assert!(!argv.contains("--model\n"));
}
