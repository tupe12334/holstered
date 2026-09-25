//! Runs the real binary with each agent's prompt payload against a mock Jev.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

const KEY: &str = "sk-or-test-secret";
const PROMPT: &str = "merge the github pull request";

struct Jev {
    url: String,
    requests: mpsc::Receiver<(String, Value)>,
}

/// Serves `status`/`body` to every request, recording (authorization, json body).
fn mock_jev(status: u16, body: Value) -> Jev {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/decisions", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut len, mut auth) = (0, String::new());
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                let (name, value) = line.split_once(": ").unwrap_or((line, ""));
                match name.to_ascii_lowercase().as_str() {
                    "content-length" => len = value.parse().unwrap(),
                    "authorization" => auth = value.to_owned(),
                    _ => {}
                }
            }
            let mut buf = vec![0; len];
            reader.read_exact(&mut buf).unwrap();
            let _ = tx.send((auth, serde_json::from_slice(&buf).unwrap_or(Value::Null)));
            let payload = body.to_string();
            let mut stream = stream;
            let _ = write!(
                stream,
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{payload}",
                payload.len()
            );
        }
    });
    Jev { url, requests: rx }
}

fn picks(choice: &str) -> Value {
    json!({"answers": {"skill": {"type": "choice", "choice": choice, "confidence": 0.9}}})
}

fn skills_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, desc, body) in [
        (
            "github-pr-merge",
            "Merge GitHub pull requests safely.",
            "Step one: check CI before merging.",
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

fn run(payload: &Value, jev: &Jev, key: Option<&str>) -> (String, String) {
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
    let mut child = cmd
        .env("HOLSTERED_SKILLS_DIRS", skills.path())
        .env("HOLSTERED_JEV_URL", &jev.url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "holstered must never fail the prompt");
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn claude_prompt() -> Value {
    json!({"hook_event_name": "UserPromptSubmit", "prompt": PROMPT, "session_id": "s1", "cwd": "/w"})
}

fn stdout_json(out: &str) -> Value {
    serde_json::from_str(out).unwrap()
}

#[test]
fn valid_pick_injects_the_skill_into_claude_code() {
    let jev = mock_jev(200, picks("github-pr-merge"));
    let (out, err) = run(&claude_prompt(), &jev, Some(KEY));

    let context = stdout_json(&out)["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(context.contains("SKILL SELECTED by holstered: github-pr-merge"));
    assert!(context.contains("Step one: check CI before merging."));

    let (auth, body) = jev.requests.recv().unwrap();
    assert_eq!(auth, format!("Bearer {KEY}"));
    assert_eq!(body["model"], "~typesafe/jev-latest");
    assert_eq!(body["state"]["user_prompt"], PROMPT);
    let criteria = body["questions"]["skill"]["criteria"].as_object().unwrap();
    assert!(criteria.contains_key("none"));
    assert!(criteria.contains_key("github-pr-merge"));
    assert!(
        !criteria.contains_key("cooking"),
        "BM25 should not shortlist unrelated skills"
    );
    assert!(
        !out.contains(KEY) && !err.contains(KEY),
        "the key must never be printed"
    );
}

#[test]
fn none_leaves_the_prompt_untouched() {
    let jev = mock_jev(200, picks("none"));
    let (out, _) = run(&claude_prompt(), &jev, Some(KEY));
    assert_eq!(stdout_json(&out), json!({}));
}

#[test]
fn unknown_skill_leaves_the_prompt_untouched() {
    let jev = mock_jev(200, picks("not-offered"));
    let (out, _) = run(&claude_prompt(), &jev, Some(KEY));
    assert_eq!(stdout_json(&out), json!({}));
}

#[test]
fn api_error_leaves_the_prompt_untouched() {
    let jev = mock_jev(500, json!({"error": "boom"}));
    let (out, err) = run(&claude_prompt(), &jev, Some(KEY));
    assert_eq!(stdout_json(&out), json!({}));
    assert!(!err.contains(KEY));
}

#[test]
fn missing_key_skips_jev() {
    let jev = mock_jev(200, picks("github-pr-merge"));
    let (out, _) = run(&claude_prompt(), &jev, None);
    assert_eq!(stdout_json(&out), json!({}));
    assert!(jev.requests.try_recv().is_err(), "no key means no API call");
}

#[test]
fn tool_events_are_approved_without_calling_jev() {
    let jev = mock_jev(200, picks("github-pr-merge"));
    let payload = json!({"hook_event_name": "PreToolUse", "tool_name": "Bash",
        "tool_input": {"command": "ls"}, "session_id": "s1"});
    let (out, _) = run(&payload, &jev, Some(KEY));
    assert_eq!(
        stdout_json(&out)["hookSpecificOutput"]["permissionDecision"],
        "allow"
    );
    assert!(jev.requests.try_recv().is_err());
}

#[test]
fn garbage_stdin_is_a_silent_no_op() {
    let jev = mock_jev(200, picks("github-pr-merge"));
    let (out, _) = run(&json!("not an object"), &jev, Some(KEY));
    // polyhook parses any JSON; a non-object is an unknown-caller notification.
    assert!(out.is_empty() || stdout_json(&out) == json!({}));
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
        let jev = mock_jev(200, picks("github-pr-merge"));
        let (out, _) = run(&payload, &jev, Some(KEY));
        let context = stdout_json(&out)
            .pointer(pointer)
            .and_then(Value::as_str)
            .map(str::to_owned);
        assert!(
            context.is_some_and(|c| c.contains("Step one: check CI before merging.")),
            "{agent}: expected skill at {pointer}, got {out}"
        );
        let (_, body) = jev.requests.recv().unwrap();
        assert_eq!(
            body["state"]["user_prompt"], PROMPT,
            "{agent}: prompt not extracted"
        );
    }
}
