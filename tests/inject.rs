//! A valid Jev pick injects the skill, and the request carries what Jev needs.

mod common;

use common::{claude_prompt, picks, run, KEY, PROMPT, SKILL_BODY};
use mockito::{Matcher, Server};
use serde_json::{json, Value};

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
