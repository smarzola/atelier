use serde_json::json;

#[test]
fn appending_thread_events_creates_monotonic_event_stream() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "thread-example";

    let first = atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "job_started",
        json!({"status":"running"}),
    )
    .expect("append first event");
    let second = atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "job_status_changed",
        json!({"status":"succeeded"}),
    )
    .expect("append second event");

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);

    let events_path = project
        .join(".atelier/threads")
        .join(thread_id)
        .join("events.jsonl");
    assert!(events_path.exists());

    let all = atelier_core::thread_events::read_thread_events(project, thread_id, 0)
        .expect("read all events");
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].kind, "job_started");
    assert_eq!(all[1].payload["status"], "succeeded");

    let later = atelier_core::thread_events::read_thread_events(project, thread_id, 1)
        .expect("read later events");
    assert_eq!(later.len(), 1);
    assert_eq!(later[0].sequence, 2);
}
