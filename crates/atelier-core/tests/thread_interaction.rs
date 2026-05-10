use serde_json::json;

#[test]
fn decides_to_answer_single_pending_prompt_before_starting_work() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "thread-example";
    let job_dir = project.join(".atelier/jobs/job-waiting");
    std::fs::create_dir_all(job_dir.join("prompts")).expect("create prompt dir");
    std::fs::write(
        job_dir.join("status.json"),
        json!({"id":"job-waiting","status":"waiting-for-prompt","thread_id":thread_id,"person":"alice","dry_run":false}).to_string(),
    )
    .expect("write status");
    std::fs::write(
        job_dir.join("prompts/prompt-1.json"),
        json!({"id":"prompt-1","summary":"Approve command","codex_request_id":"7","kind":"command"}).to_string(),
    )
    .expect("write prompt");

    let decision =
        atelier_core::thread_interaction::decide_thread_interaction(project, thread_id, "approve")
            .expect("decision");

    assert_eq!(
        decision,
        atelier_core::thread_interaction::ThreadInteractionDecision::AnswerPrompt {
            prompt_id: "prompt-1".to_string(),
        }
    );
}

#[test]
fn decides_to_queue_when_thread_has_running_job() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "thread-example";
    let job_dir = project.join(".atelier/jobs/job-running");
    std::fs::create_dir_all(&job_dir).expect("create job dir");
    std::fs::write(
        job_dir.join("status.json"),
        json!({"id":"job-running","status":"running","thread_id":thread_id,"person":"alice","dry_run":false}).to_string(),
    )
    .expect("write status");

    let decision = atelier_core::thread_interaction::decide_thread_interaction(
        project,
        thread_id,
        "Add one more thing",
    )
    .expect("decision");

    assert_eq!(
        decision,
        atelier_core::thread_interaction::ThreadInteractionDecision::QueueForRunningJob {
            job_id: "job-running".to_string(),
        }
    );
}

#[test]
fn decides_to_continue_when_thread_has_session_lineage_and_is_idle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();
    let thread_id = "thread-example";
    atelier_core::thread::append_codex_session_lineage(
        project,
        thread_id,
        json!({"codex_thread_id":"codex-thread-example"}),
    )
    .expect("append lineage");

    let decision =
        atelier_core::thread_interaction::decide_thread_interaction(project, thread_id, "Continue")
            .expect("decision");

    assert_eq!(
        decision,
        atelier_core::thread_interaction::ThreadInteractionDecision::ContinueSession {
            codex_session_id: "codex-thread-example".to_string(),
        }
    );
}

#[test]
fn decides_to_start_job_when_thread_is_idle_without_lineage() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path();

    let decision = atelier_core::thread_interaction::decide_thread_interaction(
        project,
        "thread-example",
        "Start something",
    )
    .expect("decision");

    assert_eq!(
        decision,
        atelier_core::thread_interaction::ThreadInteractionDecision::StartJob
    );
}
