use serde_json::json;

#[test]
fn appending_user_message_creates_openai_style_item_log() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();

    let item = atelier_core::thread_items::append_user_message_item(
        project,
        "thread-example",
        "alice",
        "api",
        "Please update the docs.",
        json!({"custom": "value"}),
    )
    .expect("append user item");

    assert_eq!(item.object, "conversation.item");
    assert_eq!(item.sequence, 1);
    assert_eq!(item.item_type, "message");
    assert_eq!(item.role, "user");
    assert_eq!(item.content.len(), 1);
    assert_eq!(item.content[0].content_type, "input_text");
    assert_eq!(item.content[0].text, "Please update the docs.");
    assert_eq!(
        item.metadata.get("thread").and_then(|value| value.as_str()),
        Some("thread-example")
    );
    assert_eq!(
        item.metadata.get("person").and_then(|value| value.as_str()),
        Some("alice")
    );
    assert_eq!(
        item.metadata.get("source").and_then(|value| value.as_str()),
        Some("api")
    );
    assert_eq!(
        item.metadata.get("custom").and_then(|value| value.as_str()),
        Some("value")
    );

    let item_log = project.join(".atelier/threads/thread-example/items.jsonl");
    assert!(item_log.is_file(), "items.jsonl should exist");
}

#[test]
fn thread_item_sequences_are_monotonic_and_read_after_filters() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();

    let first = atelier_core::thread_items::append_user_message_item(
        project,
        "thread-example",
        "alice",
        "api",
        "First",
        json!({}),
    )
    .expect("append first");
    let second = atelier_core::thread_items::append_assistant_message_item(
        project,
        "thread-example",
        "Second",
        json!({"job_id": "job-example"}),
    )
    .expect("append second");

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);

    let later = atelier_core::thread_items::read_thread_items(project, "thread-example", 1)
        .expect("read later items");

    assert_eq!(later.len(), 1);
    assert_eq!(later[0].id, second.id);
    assert_eq!(later[0].content[0].content_type, "output_text");
    assert_eq!(later[0].content[0].text, "Second");
}

#[test]
fn thread_item_json_uses_conversation_item_field_names() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();

    atelier_core::thread_items::append_user_message_item(
        project,
        "thread-example",
        "alice",
        "api",
        "Hello",
        json!({}),
    )
    .expect("append item");

    let raw = std::fs::read_to_string(project.join(".atelier/threads/thread-example/items.jsonl"))
        .expect("read item log");
    let line: serde_json::Value =
        serde_json::from_str(raw.lines().next().expect("one line")).expect("parse item json");

    assert_eq!(
        line.get("object").and_then(|value| value.as_str()),
        Some("conversation.item")
    );
    assert_eq!(
        line.get("type").and_then(|value| value.as_str()),
        Some("message")
    );
    assert_eq!(
        line.pointer("/content/0/type")
            .and_then(|value| value.as_str()),
        Some("input_text")
    );
    assert!(
        line.get("item_type").is_none(),
        "JSON must use OpenAI-style type field"
    );
    assert!(
        line.pointer("/content/0/content_type").is_none(),
        "content JSON must use type field"
    );
}
