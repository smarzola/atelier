use serde_json::json;

#[test]
fn delivery_cursors_return_only_undelivered_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "thread-example";

    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "agent_message_snapshot",
        json!({"text":"one"}),
    )
    .expect("append one");
    atelier_core::thread_events::append_thread_event(
        project,
        thread_id,
        Some("job-one"),
        "final_result",
        json!({"text":"two"}),
    )
    .expect("append two");

    let first =
        atelier_core::thread_delivery::read_undelivered_events(project, thread_id, "cli-follow")
            .expect("read first");
    assert_eq!(first.len(), 2);

    atelier_core::thread_delivery::advance_delivery_cursor(project, thread_id, "cli-follow", 1)
        .expect("advance cursor");

    let second =
        atelier_core::thread_delivery::read_undelivered_events(project, thread_id, "cli-follow")
            .expect("read second");
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].sequence, 2);

    atelier_core::thread_delivery::advance_delivery_cursor(project, thread_id, "cli-follow", 2)
        .expect("advance cursor again");
    let empty =
        atelier_core::thread_delivery::read_undelivered_events(project, thread_id, "cli-follow")
            .expect("read empty");
    assert!(empty.is_empty());
}
