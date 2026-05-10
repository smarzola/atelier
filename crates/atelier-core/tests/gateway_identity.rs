use atelier_core::gateway::{GatewayMessageEvent, bind_person, resolve_person};

#[test]
fn binds_external_gateway_users_to_people_in_global_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    unsafe {
        std::env::set_var("HOME", temp.path());
    }

    let binding = bind_person("example-gateway", "external-user", "alice").expect("bind person");
    assert_eq!(binding.gateway, "example-gateway");
    assert_eq!(binding.external_user, "external-user");
    assert_eq!(binding.person, "alice");

    let resolved = resolve_person("example-gateway", "external-user")
        .expect("resolve person")
        .expect("person binding");
    assert_eq!(resolved.person, "alice");
}

#[test]
fn message_event_shape_is_gateway_neutral() {
    let event: GatewayMessageEvent = serde_json::from_str(
        r#"{
  "gateway": "example-gateway",
  "external_thread": "external-thread",
  "external_user": "external-user",
  "project": "example-project",
  "thread": "thread-example",
  "person": "alice",
  "text": "Do the thing"
}"#,
    )
    .expect("parse event");

    assert_eq!(event.gateway, "example-gateway");
    assert_eq!(event.external_thread.as_deref(), Some("external-thread"));
    assert_eq!(event.external_user.as_deref(), Some("external-user"));
    assert_eq!(event.project.as_deref(), Some("example-project"));
    assert_eq!(event.thread.as_deref(), Some("thread-example"));
    assert_eq!(event.person.as_deref(), Some("alice"));
    assert_eq!(event.text, "Do the thing");
}
