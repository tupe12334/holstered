//! Every failure mode leaves the prompt untouched.

mod common;

use common::{claude_prompt, mock_jev, picks, run, KEY};
use mockito::Server;
use serde_json::json;

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
