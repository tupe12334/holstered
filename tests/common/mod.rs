//! Shared by the end-to-end tests: run the real binary against a mock Jev.
// Each test file compiles its own copy and uses only some helpers.
#![allow(dead_code, unused_imports)]

mod agents;
mod backend;
mod fixtures;
mod run;

pub use agents::agent_cases;
pub use backend::{run, run_kev};
pub use fixtures::skills_dir;

use mockito::{Mock, ServerGuard};
use serde_json::{json, Value};

pub const KEY: &str = "sk-or-test-secret";
pub const PROMPT: &str = "merge the github pull request";
pub const SKILL_BODY: &str = "Step one: check CI before merging.";

pub fn picks(choice: &str) -> String {
    json!({"answers": {"skill": {"type": "choice", "choice": choice, "confidence": 0.9}}})
        .to_string()
}

/// A Jev endpoint that answers `body` with `status`, expecting `hits` calls.
pub fn mock_jev(server: &mut ServerGuard, status: usize, body: String, hits: usize) -> Mock {
    server
        .mock("POST", "/decisions")
        .with_status(status)
        .with_header("content-type", "application/json")
        .with_body(body)
        .expect(hits)
        .create()
}

pub fn claude_prompt() -> Value {
    json!({"hook_event_name": "UserPromptSubmit", "prompt": PROMPT, "session_id": "s1", "cwd": "/w"})
}
