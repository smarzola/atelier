use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn skill_add_project_copies_skill_into_codex_native_skills_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    let source_skill = temp.path().join("triage-inbox");
    std::fs::create_dir(&source_skill).expect("create source skill");
    std::fs::write(source_skill.join("SKILL.md"), "# Triage inbox\n").expect("write skill");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "skill",
            "add",
            "project",
            project.to_str().expect("utf8 path"),
            source_skill.to_str().expect("utf8 path"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added project skill triage-inbox"));

    assert_eq!(
        std::fs::read_to_string(project.join(".agents/skills/triage-inbox/SKILL.md"))
            .expect("read copied skill"),
        "# Triage inbox\n"
    );
}

#[test]
fn mcp_add_project_writes_codex_native_project_config() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    std::fs::create_dir(&project).expect("create project");

    Command::cargo_bin("atelier")
        .expect("atelier binary exists")
        .args([
            "mcp",
            "add",
            "project",
            project.to_str().expect("utf8 path"),
            "time",
            "--",
            "uvx",
            "mcp-server-time",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added project MCP server time"));

    let config =
        std::fs::read_to_string(project.join(".codex/config.toml")).expect("read codex config");
    assert!(config.contains("[mcp_servers.time]"));
    assert!(config.contains("command = \"uvx\""));
    assert!(config.contains("args = [\"mcp-server-time\"]"));
}
