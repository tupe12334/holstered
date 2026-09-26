//! `HOLSTERED_TIMEOUT_MS` bounds the wait, e.g. to ride out Kev's warm-up.

mod common;

use common::{claude_prompt, exec, picks};
use mockito::{Server, ServerGuard};
use serde_json::{json, Value};
use std::{thread, time::Duration};

/// Kev that answers after 300 ms, run with `timeout_ms`.
fn slow_kev(timeout_ms: &str) -> (Value, ServerGuard) {
    let mut server = Server::new();
    server
        .mock("POST", "/decisions")
        .with_chunked_body(|w| {
            thread::sleep(Duration::from_millis(300));
            w.write_all(picks("github-pr-merge").as_bytes())
        })
        .create();
    let env = [
        ("HOLSTERED_KEV_URL", format!("{}/decisions", server.url())),
        ("HOLSTERED_TIMEOUT_MS", timeout_ms.to_owned()),
    ];
    (exec(&claude_prompt(), &env), server)
}

#[test]
fn a_short_timeout_gives_up_and_leaves_the_prompt_untouched() {
    assert_eq!(slow_kev("50").0, json!({}));
}

#[test]
fn a_long_enough_timeout_waits_for_the_pick() {
    let (out, _server) = slow_kev("5000");
    assert!(out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("SKILL SELECTED by holstered: github-pr-merge"));
}
