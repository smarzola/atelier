use atelier_core::thread_pending::{
    PendingThreadInteraction, clear_pending_interaction, pending_interaction_path,
    read_pending_interaction, write_pending_interaction,
};

#[test]
fn pending_interaction_round_trips_and_clears() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("example-project");
    std::fs::create_dir_all(&project).expect("project dir");

    let pending = PendingThreadInteraction {
        kind: "approval_request".to_string(),
        item_id: "item-approval".to_string(),
        job_id: "job-example".to_string(),
        prompt_id: "prompt-example".to_string(),
        choices: vec![
            "approve".to_string(),
            "decline".to_string(),
            "cancel".to_string(),
        ],
    };

    write_pending_interaction(&project, "thread-example", &pending).expect("write pending");
    assert!(pending_interaction_path(&project, "thread-example").is_file());

    let read = read_pending_interaction(&project, "thread-example")
        .expect("read pending")
        .expect("pending exists");
    assert_eq!(read, pending);

    clear_pending_interaction(&project, "thread-example").expect("clear pending");
    assert!(
        read_pending_interaction(&project, "thread-example")
            .expect("read cleared pending")
            .is_none()
    );
}
