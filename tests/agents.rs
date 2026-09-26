//! Each prompt-hook agent gets the skill in its own output format.

mod common;

use common::{agent_cases, picks, run, KEY, PROMPT, SKILL_BODY};
use mockito::{Matcher, Server};
use serde_json::{json, Value};

#[test]
fn every_prompt_hook_agent_gets_its_own_format() {
    for (agent, payload, pointer) in agent_cases() {
        let mut server = Server::new();
        let jev = server
            .mock("POST", "/decisions")
            .match_body(Matcher::PartialJson(
                json!({"state": {"user_prompt": PROMPT}}),
            ))
            .with_body(picks("github-pr-merge"))
            .create();
        let out = run(&payload, &server, Some(KEY));
        jev.assert();
        let context = out
            .pointer(pointer)
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(
            context.contains(SKILL_BODY),
            "{agent}: expected skill at {pointer}, got {out}"
        );
    }
}
