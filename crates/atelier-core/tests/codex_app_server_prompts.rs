use atelier_core::codex_app_server::{PendingPromptStatus, parse_pending_prompt};

#[test]
fn parses_command_approval_prompt() {
    let line = r#"{"method":"item/commandExecution/requestApproval","id":7,"params":{"threadId":"thread-example","turnId":"turn-example","itemId":"call-example","reason":"Need to run tests","command":"cargo test","cwd":"/workspace/example","availableDecisions":["accept",{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["cargo","test"]}},"cancel"]}}"#;

    let prompt = parse_pending_prompt(line).expect("command prompt");

    assert_eq!(prompt.id, "prompt-7");
    assert_eq!(prompt.codex_request_id, "7");
    assert_eq!(prompt.method, "item/commandExecution/requestApproval");
    assert_eq!(prompt.codex_thread_id.as_deref(), Some("thread-example"));
    assert_eq!(prompt.codex_turn_id.as_deref(), Some("turn-example"));
    assert_eq!(prompt.codex_item_id.as_deref(), Some("call-example"));
    assert_eq!(prompt.status, PendingPromptStatus::Pending);
    assert_eq!(prompt.summary, "Approve command: cargo test");
    assert_eq!(
        prompt.available_decisions,
        vec!["accept", "acceptWithExecpolicyAmendment", "cancel"]
    );
    assert_eq!(prompt.params["reason"], "Need to run tests");
}

#[test]
fn parses_file_permission_tool_and_mcp_prompts() {
    let cases = [
        (
            r#"{"method":"item/fileChange/requestApproval","id":"file-1","params":{"threadId":"thread-example","turnId":"turn-example","itemId":"patch-example","reason":"Update docs","availableDecisions":["accept","decline","cancel"]}}"#,
            "prompt-file-1",
            "Approve file changes: Update docs",
        ),
        (
            r#"{"method":"item/permissions/requestApproval","id":8,"params":{"threadId":"thread-example","turnId":"turn-example","itemId":"perm-example","reason":"Need network access"}}"#,
            "prompt-8",
            "Approve permissions: Need network access",
        ),
        (
            r#"{"method":"item/tool/requestUserInput","id":9,"params":{"threadId":"thread-example","turnId":"turn-example","itemId":"tool-example","questions":[{"id":"q1","header":"Choice","question":"Pick one","options":[{"label":"A","description":"Option A"}]}]}}"#,
            "prompt-9",
            "Answer tool user-input prompt",
        ),
        (
            r#"{"method":"mcpServer/elicitation/request","id":10,"params":{"threadId":"thread-example","turnId":"turn-example","serverName":"example-mcp","mode":"form","message":"Provide a value"}}"#,
            "prompt-10",
            "Answer MCP elicitation from example-mcp: Provide a value",
        ),
    ];

    for (line, expected_id, expected_summary) in cases {
        let prompt = parse_pending_prompt(line).expect("pending prompt");
        assert_eq!(prompt.id, expected_id);
        assert_eq!(prompt.summary, expected_summary);
        assert_eq!(prompt.status, PendingPromptStatus::Pending);
        assert_eq!(prompt.codex_thread_id.as_deref(), Some("thread-example"));
    }
}

#[test]
fn ignores_notifications_that_are_not_pending_prompts() {
    let line = r#"{"method":"turn/completed","params":{"threadId":"thread-example"}}"#;

    assert_eq!(parse_pending_prompt(line), None);
}
