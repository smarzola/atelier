use serde_json::json;
use tempfile::tempdir;

#[test]
fn bounded_progress_selects_prompt_latest_snapshot_and_final_result() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "support-thread";

    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "job_started",
        json!({"status":"running"}),
    )
    .expect("append started");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "agent_message_snapshot",
        json!({"text":"first draft"}),
    )
    .expect("append first snapshot");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "agent_message_snapshot",
        json!({"text":"latest draft"}),
    )
    .expect("append latest snapshot");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "prompt_required",
        json!({"prompt_id":"prompt-1","summary":"Approve command"}),
    )
    .expect("append prompt");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "job_succeeded",
        json!({"status":"succeeded"}),
    )
    .expect("append succeeded");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "final_result",
        json!({"text":"done"}),
    )
    .expect("append final");

    let events = atelier_core::thread_events::read_thread_events(project, thread_id, 0)
        .expect("read events");
    let progress = atelier_core::thread_progress::select_bounded_progress_events(&events);
    let kinds = progress
        .iter()
        .map(|event| event.kind.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        vec!["agent_message_snapshot", "prompt_required", "final_result"]
    );
    assert_eq!(progress[0].payload["text"], "latest draft");
    assert_eq!(progress[1].payload["prompt_id"], "prompt-1");
    assert_eq!(progress[2].payload["text"], "done");
}

#[test]
fn bounded_progress_does_not_emit_snapshots_after_final_result() {
    let temp = tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "complete-thread";

    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "agent_message_snapshot",
        json!({"text":"summary"}),
    )
    .expect("append snapshot");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "final_result",
        json!({"text":"summary"}),
    )
    .expect("append final");

    let events = atelier_core::thread_events::read_thread_events(project, thread_id, 0)
        .expect("read events");
    let progress = atelier_core::thread_progress::select_bounded_progress_events(&events);

    assert_eq!(progress.len(), 1);
    assert_eq!(progress[0].kind, "final_result");
}
