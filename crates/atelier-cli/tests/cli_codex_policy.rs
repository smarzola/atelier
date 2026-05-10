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
            "Dogfood",
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
fn work_dry_run_exposes_codex_policy_overrides_without_mutating_project_config() {
    let (_temp, project, thread_id) = initialized_project();
    let codex_config = project.join(".codex/config.toml");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "work",
            project.to_str().expect("utf8 path"),
            "--thread",
            &thread_id,
            "--as",
            "alice",
            "--approval-policy",
            "on-request",
            "--sandbox",
            "workspace-write",
            "--model",
            "gpt-5.1-codex-max",
            "--search",
            "--dry-run",
            "Check the next dogfood gap",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("codex exec")
                .and(predicate::str::contains(
                    "-c approval_policy=\"on-request\"",
                ))
                .and(predicate::str::contains("--sandbox workspace-write"))
                .and(predicate::str::contains("--model gpt-5.1-codex-max"))
                .and(predicate::str::contains("--search"))
                .and(predicate::str::contains("Check the next dogfood gap")),
        );

    assert!(
        !codex_config.exists(),
        "invocation-time policy must not rewrite project Codex config"
    );
}

#[test]
fn work_passes_codex_policy_overrides_to_fake_codex_and_records_them() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = project.join("codex-policy-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho 'FAKE_POLICY_RESULT'\n",
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
            "--approval-policy",
            "never",
            "--sandbox",
            "read-only",
            "--search",
            "Summarize without writes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAKE_POLICY_RESULT"));

    let argv = std::fs::read_to_string(&recorder).expect("read codex argv");
    assert!(argv.contains("exec\n"));
    assert!(argv.contains("-c\napproval_policy=\"never\"\n"));
    assert!(argv.contains("--sandbox\nread-only\n"));
    assert!(argv.contains("--search\n"));
    assert!(argv.contains("--cd\n"));
    assert!(argv.contains("Summarize without writes"));

    let jobs_dir = project.join(".atelier/jobs");
    let mut job_dirs: Vec<_> = std::fs::read_dir(&jobs_dir)
        .expect("read jobs dir")
        .map(|entry| entry.expect("job entry").path())
        .collect();
    job_dirs.sort();
    assert_eq!(job_dirs.len(), 1);
    let status = std::fs::read_to_string(job_dirs[0].join("status.json")).expect("read status");
    assert!(status.contains("\"-c\""));
    assert!(status.contains("\"approval_policy=\\\"never\\\"\""));
    assert!(status.contains("\"--sandbox\""));
    assert!(status.contains("\"read-only\""));
    assert!(status.contains("\"--search\""));
}

#[test]
fn continue_passes_codex_policy_overrides_to_fake_codex() {
    let (_temp, project, thread_id) = initialized_project();
    let fake_bin = project.join("fake-bin");
    std::fs::create_dir(&fake_bin).expect("create fake bin");
    let fake_codex = fake_bin.join("codex");
    let recorder = project.join("codex-continue-policy-argv.txt");
    std::fs::write(
        &fake_codex,
        format!(
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" > {}\necho 'FAKE_CONTINUE_POLICY_RESULT'\n",
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
            "--approval-policy",
            "on-request",
            "--model",
            "gpt-5.1-codex-max",
            "Continue carefully",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAKE_CONTINUE_POLICY_RESULT"));

    let argv = std::fs::read_to_string(&recorder).expect("read codex argv");
    assert!(argv.contains("exec\n"));
    assert!(argv.contains("resume\n"));
    assert!(argv.contains("--last\n"));
    assert!(argv.contains("-c\napproval_policy=\"on-request\"\n"));
    assert!(argv.contains("--model\ngpt-5.1-codex-max\n"));
    assert!(argv.contains("Continue carefully"));
}
