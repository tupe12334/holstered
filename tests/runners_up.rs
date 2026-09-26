//! Runners-up over the runner-up threshold follow the pick, on Jev and Kev.

mod common;

use common::{jev_then_kev_contexts, SKILL_BODY};

#[test]
fn runner_up_over_threshold_is_injected_second() {
    for context in jev_then_kev_contexts(Some("0.2")) {
        let merge = context.find("SKILL SELECTED by holstered: github-pr-merge");
        let review = context.find("SKILL SELECTED by holstered: github-pr-review");
        assert!(merge.unwrap() < review.unwrap());
        assert!(context.contains(SKILL_BODY) && context.contains("Review body."));
    }
}

#[test]
fn unset_or_raised_threshold_means_no_runner_up() {
    for context in [
        jev_then_kev_contexts(None),
        jev_then_kev_contexts(Some("0.5")),
    ]
    .concat()
    {
        assert!(context.contains("github-pr-merge"));
        assert!(!context.contains("github-pr-review"));
    }
}
