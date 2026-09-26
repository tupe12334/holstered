//! Runners-up at or over the runner-up threshold follow the pick; unset, none do.

mod common;

use common::{claude_prompt, exec, mock_jev, KEY, SKILL_BODY};
use mockito::Server;
use serde_json::json;

fn answer() -> String {
    json!({"answers": {"skill": {"type": "choice", "choice": "github-pr-merge",
        "probabilities": {"github-pr-merge": 0.7, "github-pr-review": 0.25, "none": 0.05}}}})
    .to_string()
}

fn run(server: &Server, runner_up_threshold: Option<&str>) -> String {
    let mut env = vec![
        ("HOLSTERED_JEV_URL", format!("{}/decisions", server.url())),
        ("OPENROUTER_API_KEY", KEY.to_owned()),
    ];
    env.extend(runner_up_threshold.map(|t| ("HOLSTERED_RUNNER_UP_THRESHOLD", t.to_owned())));
    let out = exec(&claude_prompt(), &env);
    out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn runner_up_over_threshold_is_injected_second() {
    let mut server = Server::new();
    let jev = mock_jev(&mut server, 200, answer(), 1);
    let context = run(&server, Some("0.2"));
    jev.assert();
    let merge = context.find("SKILL SELECTED by holstered: github-pr-merge");
    let review = context.find("SKILL SELECTED by holstered: github-pr-review");
    assert!(merge.unwrap() < review.unwrap());
    assert!(context.contains(SKILL_BODY) && context.contains("Review body."));
}

#[test]
fn unset_or_raised_threshold_means_no_runner_up() {
    for runner_up_threshold in [None, Some("0.5")] {
        let mut server = Server::new();
        let jev = mock_jev(&mut server, 200, answer(), 1);
        let context = run(&server, runner_up_threshold);
        jev.assert();
        assert!(context.contains("github-pr-merge"));
        assert!(!context.contains("github-pr-review"));
    }
}
