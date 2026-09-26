//! A local Kev server stands in for Jev: same request, no key needed.

mod common;

use common::{claude_prompt, picks, run_kev, KEY, SKILL_BODY};
use mockito::{Matcher, Server};
use serde_json::json;

#[test]
fn kev_picks_without_a_key_and_sends_no_auth_header() {
    let mut server = Server::new();
    let kev = server
        .mock("POST", "/decisions")
        .match_header("authorization", Matcher::Missing)
        .match_body(Matcher::PartialJson(
            json!({"model": "kev-latest", "state": {"user_prompt": common::PROMPT}}),
        ))
        .with_body(picks("github-pr-merge"))
        .create();

    let out = run_kev(&claude_prompt(), &server);

    kev.assert();
    let context = out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("SKILL SELECTED by holstered: github-pr-merge"));
    assert!(context.contains(SKILL_BODY));
    assert!(!context.contains(KEY));
}

#[test]
fn kev_down_leaves_the_prompt_untouched() {
    let mut server = Server::new();
    let kev = server.mock("POST", "/decisions").with_status(503).create();
    assert_eq!(run_kev(&claude_prompt(), &server), json!({}));
    kev.assert();
}
