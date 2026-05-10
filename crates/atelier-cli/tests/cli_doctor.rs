use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn doctor_reports_missing_codex_binary_when_path_does_not_contain_codex() {
    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("PATH", "")
        .arg("doctor")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Codex binary: missing"))
        .stdout(predicate::str::contains("codex not found on PATH"));
}

#[test]
fn doctor_checks_project_scaffold_when_project_is_provided() {
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

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("PATH", "")
        .args(["doctor", "--project", project.to_str().expect("utf8 path")])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Codex binary: missing"))
        .stdout(predicate::str::contains("Project path: ok"))
        .stdout(predicate::str::contains("Project manifest: ok"))
        .stdout(predicate::str::contains("Project instructions: ok"))
        .stdout(predicate::str::contains("Threads directory: ok"));
}

#[test]
fn doctor_reports_fake_codex_capabilities() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir(&bin_dir).expect("create bin dir");
    let fake_codex = bin_dir.join("codex");
    std::fs::write(
        &fake_codex,
        "#!/usr/bin/env sh\ncase \"$1\" in\n  --version) echo 'codex-cli 0.0.0-test' ;;\n  exec) echo 'Usage: codex exec [OPTIONS] [PROMPT]' ;;\n  resume) echo 'Usage: codex resume [OPTIONS] [SESSION]' ;;\n  *) echo 'fake codex' ;;\nesac\n",
    )
    .expect("write fake codex");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake_codex, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }

    let original_path = std::env::var_os("PATH").expect("PATH is set");
    let test_path = std::env::join_paths(
        std::iter::once(bin_dir.as_os_str().to_owned())
            .chain(std::env::split_paths(&original_path).map(|path| path.into_os_string())),
    )
    .expect("join PATH");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .env("PATH", test_path)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Codex binary: ok"))
        .stdout(predicate::str::contains("codex-cli 0.0.0-test"))
        .stdout(predicate::str::contains("Codex exec: ok"))
        .stdout(predicate::str::contains("Codex resume: ok"));
}
