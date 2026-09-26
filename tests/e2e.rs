//! Runs the real binary with each agent's prompt payload against a mock Jev.

use assert_cmd::Command;
use mockito::{Matcher, Mock, Server, ServerGuard};
use serde_json::{json, Value};

const KEY: &str = "sk-or-test-secret";
const PROMPT: &str = "merge the github pull request";
const SKILL_BODY: &str = "Step one: check CI before merging.";

fn picks(choice: &str) -> String {
    json!({"answers": {"skill": {"type": "choice", "choice": choice, "confidence": 0.9}}})
        .to_string()
}

/// A Jev endpoint that answers `body` with `status`, expecting `hits` calls.
fn mock_jev(server: &mut ServerGuard, status: usize, body: String, hits: usize) -> Mock {
    server
        .mock("POST", "/decisions")
        .with_status(status)
        .with_header("content-type", "application/json")
        .with_body(body)
        .expect(hits)
        .create()
}

fn skills_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, desc, body) in [
        (
            "github-pr-merge",
            "Merge GitHub pull requests safely.",
            SKILL_BODY,
        ),
        (
            "github-pr-review",
            "Review a GitHub pull request diff.",
            "Review body.",
        ),
        ("cooking", "Bake bread.", "Knead."),
    ] {
        let path = dir.path().join(name);
        std::fs::create_dir(&path).unwrap();
        std::fs::write(
            path.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {desc}\n---\n{body}\n"),
        )
        .unwrap();
    }
    dir
}

/// Runs holstered on `payload`; asserts it exits 0 and never prints the key.
fn run(payload: &Value, server: &ServerGuard, key: Option<&str>) -> Value {
    let skills = skills_dir();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_holstered"));
    for var in [
        "POLYHOOK_CALLER",
        "CLAUDE_CODE_VERSION",
        "CURSOR_SESSION_ID",
        "WINDSURF_SESSION_ID",
        "CLINE_SESSION_ID",
        "AMP_SESSION_ID",
        "GEMINI_PROJECT_DIR",
        "OPENROUTER_API_KEY",
    ] {
        cmd.env_remove(var);
    }
    if let Some(key) = key {
        cmd.env("OPENROUTER_API_KEY", key);
    }
    let out = cmd
        .env("HOLSTERED_SKILLS_DIRS", skills.path())
        .env("HOLSTERED_JEV_URL", format!("{}/decisions", server.url()))
        .write_stdin(payload.to_string())
        .assert()
        .success()
        .get_output()
        .clone();
    let (stdout, stderr) = (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    );
    assert!(
        !stdout.contains(KEY) && !stderr.contains(KEY),
        "the key must never be printed"
    );
    serde_json::from_str(&stdout).unwrap_or(Value::Null)
}

fn claude_prompt() -> Value {
    json!({"hook_event_name": "UserPromptSubmit", "prompt": PROMPT, "session_id": "s1", "cwd": "/w"})
}

#[test]
fn valid_pick_injects_the_skill_into_claude_code() {
    let mut server = Server::new();
    let jev = server
        .mock("POST", "/decisions")
        .match_header("authorization", format!("Bearer {KEY}").as_str())
        .match_body(Matcher::PartialJson(json!({
            "model": "~typesafe/jev-latest",
            "state": {"user_prompt": PROMPT},
        })))
        .match_request(|req| {
            let body: Value = serde_json::from_slice(req.body().unwrap()).unwrap();
            let criteria = body["questions"]["skill"]["criteria"]
                .as_object()
                .unwrap()
                .clone();
            // BM25 shortlists the PR skills and drops the unrelated one.
            criteria.contains_key("none")
                && criteria.contains_key("github-pr-merge")
                && !criteria.contains_key("cooking")
        })
        .with_body(picks("github-pr-merge"))
        .create();

    let out = run(&claude_prompt(), &server, Some(KEY));

    jev.assert();
    let context = out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("SKILL SELECTED by holstered: github-pr-merge"));
    assert!(context.contains(SKILL_BODY));
}

#[test]
fn none_leaves_the_prompt_untouched() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 200, picks("none"), 1);
    assert_eq!(run(&claude_prompt(), &server, Some(KEY)), json!({}));
    jev.assert();
}

#[test]
fn unknown_skill_leaves_the_prompt_untouched() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 200, picks("not-offered"), 1);
    assert_eq!(run(&claude_prompt(), &server, Some(KEY)), json!({}));
    jev.assert();
}

#[test]
fn api_error_leaves_the_prompt_untouched() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 500, json!({"error": "boom"}).to_string(), 1);
    assert_eq!(run(&claude_prompt(), &server, Some(KEY)), json!({}));
    jev.assert();
}

#[test]
fn missing_key_skips_jev() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 200, picks("github-pr-merge"), 0);
    assert_eq!(run(&claude_prompt(), &server, None), json!({}));
    jev.assert();
}

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

#[test]
fn every_prompt_hook_agent_gets_its_own_format() {
    let cases = [
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
    ];
    for (agent, payload, pointer) in cases {
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
