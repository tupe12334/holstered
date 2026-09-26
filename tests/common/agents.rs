use super::PROMPT;
use serde_json::{json, Value};

/// Each prompt-hook agent's payload and where its injected context lands.
pub fn agent_cases() -> Vec<(&'static str, Value, &'static str)> {
    vec![
        (
            "codex",
            json!({"hook_event_name": "UserPromptSubmit", "prompt": PROMPT, "session_id": "s1",
            "turn_id": "t1", "model": "gpt-5", "cwd": "/w"}),
            "/hookSpecificOutput/additionalContext",
        ),
        (
            "gemini-cli",
            json!({"hook_event_name": "BeforeAgent", "prompt": PROMPT, "session_id": "s1", "cwd": "/w"}),
            "/hookSpecificOutput/additionalContext",
        ),
        (
            "hermes",
            json!({"hook_event_name": "pre_llm_call", "tool_name": null, "tool_input": null,
            "session_id": "s1", "cwd": "/w", "extra": {"user_message": PROMPT}}),
            "/context",
        ),
        (
            "cline",
            json!({"clineVersion": "3.40.0", "hookName": "UserPromptSubmit", "taskId": "t",
            "workspaceRoots": ["/w"], "userPromptSubmit": {"prompt": PROMPT, "attachments": []}}),
            "/contextModification",
        ),
    ]
}
