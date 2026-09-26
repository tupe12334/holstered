//! Non-prompt input passes through without calling Jev.

mod common;

use common::{mock_jev, picks, run, KEY};
use mockito::Server;
use serde_json::json;

#[test]
fn tool_events_are_approved_without_calling_jev() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 200, picks("github-pr-merge"), 0);
    let payload = json!({"hook_event_name": "PreToolUse", "tool_name": "Bash",
        "tool_input": {"command": "ls"}, "session_id": "s1"});
    let out = run(&payload, &server, Some(KEY));
    assert_eq!(out["hookSpecificOutput"]["permissionDecision"], "allow");
    jev.assert();
}

#[test]
fn non_object_stdin_is_a_no_op() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 200, picks("github-pr-merge"), 0);
    let out = run(&json!("not an object"), &server, Some(KEY));
    assert!(out.is_null() || out == json!({}));
    jev.assert();
}
