use crate::thread_events::ThreadEvent;

pub fn select_bounded_progress_events(events: &[ThreadEvent]) -> Vec<ThreadEvent> {
    let final_event = events
        .iter()
        .rev()
        .find(|event| event.kind == "final_result");

    let mut selected = Vec::new();
    let mut latest_snapshot = None;

    for event in events {
        match event.kind.as_str() {
            "agent_message_snapshot" => latest_snapshot = Some(event.clone()),
            "prompt_required" | "queued_message_ready" => selected.push(event.clone()),
            _ => {}
        }
    }

    if let Some(snapshot) = latest_snapshot {
        selected.insert(0, snapshot);
    }

    if let Some(final_event) = final_event {
        selected.push(final_event.clone());
    }

    deduplicate_adjacent_text_events(selected)
}

fn deduplicate_adjacent_text_events(events: Vec<ThreadEvent>) -> Vec<ThreadEvent> {
    let mut deduplicated = Vec::new();
    for event in events {
        if deduplicated
            .last()
            .and_then(event_text)
            .zip(event_text(&event))
            .is_some_and(|(previous, current)| previous == current)
        {
            deduplicated.pop();
        }
        deduplicated.push(event);
    }
    deduplicated
}

fn event_text(event: &ThreadEvent) -> Option<&str> {
    event.payload.get("text")?.as_str()
}
