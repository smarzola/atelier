use serde_json::json;

#[test]
fn queues_message_for_thread_and_emits_ready_event() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "thread-example";

    let queued = atelier_core::thread_queue::queue_thread_message(
        project,
        thread_id,
        "alice",
        "Add one more detail",
    )
    .expect("queue message");

    assert_eq!(queued.sequence, 1);
    let queue_path = project
        .join(".atelier/threads")
        .join(thread_id)
        .join("queued-messages.jsonl");
    let content = std::fs::read_to_string(queue_path).expect("read queue");
    assert!(content.contains("Add one more detail"));

    atelier_core::thread_queue::mark_queued_messages_ready(project, thread_id, Some("job-one"))
        .expect("mark ready");
    let events = atelier_core::thread_events::read_thread_events(project, thread_id, 0)
        .expect("read events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "queued_message_ready");
    assert_eq!(events[0].payload["queued_sequence"], json!(1));
}
