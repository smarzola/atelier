use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingPromptStatus {
    Pending,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingPrompt {
    pub id: String,
    pub codex_request_id: String,
    pub method: String,
    pub codex_thread_id: Option<String>,
    pub codex_turn_id: Option<String>,
    pub codex_item_id: Option<String>,
    pub status: PendingPromptStatus,
    pub summary: String,
    pub available_decisions: Vec<String>,
    pub params: Value,
}

pub fn parse_pending_prompt(line: &str) -> Option<PendingPrompt> {
    let message: Value = serde_json::from_str(line).ok()?;
    let method = message.get("method")?.as_str()?;
    let request_id = message.get("id")?;
    let params = message.get("params")?.clone();

    match method {
        "item/commandExecution/requestApproval"
        | "item/fileChange/requestApproval"
        | "item/permissions/requestApproval"
        | "item/tool/requestUserInput"
        | "mcpServer/elicitation/request" => Some(PendingPrompt {
            id: format!("prompt-{}", request_id_to_string(request_id)),
            codex_request_id: request_id_to_string(request_id),
            method: method.to_string(),
            codex_thread_id: string_param(&params, "threadId"),
            codex_turn_id: string_param(&params, "turnId"),
            codex_item_id: string_param(&params, "itemId"),
            status: PendingPromptStatus::Pending,
            summary: summarize_prompt(method, &params),
            available_decisions: available_decisions(&params),
            params,
        }),
        _ => None,
    }
}

fn request_id_to_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => value.to_string(),
    }
}

fn string_param(params: &Value, key: &str) -> Option<String> {
    params.get(key)?.as_str().map(ToString::to_string)
}

fn summarize_prompt(method: &str, params: &Value) -> String {
    match method {
        "item/commandExecution/requestApproval" => {
            let command = string_param(params, "command").unwrap_or_else(|| "command".to_string());
            format!("Approve command: {command}")
        }
        "item/fileChange/requestApproval" => {
            let reason =
                string_param(params, "reason").unwrap_or_else(|| "file changes".to_string());
            format!("Approve file changes: {reason}")
        }
        "item/permissions/requestApproval" => {
            let reason = string_param(params, "reason")
                .unwrap_or_else(|| "additional permissions".to_string());
            format!("Approve permissions: {reason}")
        }
        "item/tool/requestUserInput" => "Answer tool user-input prompt".to_string(),
        "mcpServer/elicitation/request" => {
            let server =
                string_param(params, "serverName").unwrap_or_else(|| "MCP server".to_string());
            let message =
                string_param(params, "message").unwrap_or_else(|| "elicitation".to_string());
            format!("Answer MCP elicitation from {server}: {message}")
        }
        _ => "Answer Codex prompt".to_string(),
    }
}

fn available_decisions(params: &Value) -> Vec<String> {
    params
        .get("availableDecisions")
        .and_then(Value::as_array)
        .map(|values| values.iter().map(decision_to_string).collect())
        .unwrap_or_default()
}

fn decision_to_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Object(map) => map
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| value.to_string()),
        _ => value.to_string(),
    }
}
