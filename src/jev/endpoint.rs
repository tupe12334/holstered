//! Where the decision goes: local Kev when configured, else Jev on OpenRouter.

use super::{DEFAULT_URL, JEV_MODEL, JEV_RUNNER_UP, KEV_MODEL, KEV_RUNNER_UP};
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 8000;

/// A decisions endpoint, its model and key, how long to wait for its answer
/// (`HOLSTERED_TIMEOUT_MS`, default 8s), and the probability a runner-up
/// needs (`HOLSTERED_RUNNER_UP_THRESHOLD`; default 0.2 on Jev, 0.15 on Kev), and the
/// probability the pick needs (`HOLSTERED_PICK_THRESHOLD`; unset, any).
pub struct Endpoint {
    pub url: String,
    pub model: &'static str,
    pub key: Option<String>,
    pub timeout: Duration,
    pub pick_threshold: Option<f64>,
    pub runner_up_threshold: f64,
}

/// `HOLSTERED_KEV_URL` selects a local Kev server, which needs no key.
/// Otherwise Jev needs `OPENROUTER_API_KEY`; `None` when it is unset.
pub fn endpoint() -> Option<Endpoint> {
    let (url, model, key, runner_up_threshold) = match var("HOLSTERED_KEV_URL") {
        Some(url) => (url, KEV_MODEL, None, KEV_RUNNER_UP),
        None => (
            var("HOLSTERED_JEV_URL").unwrap_or_else(|| DEFAULT_URL.into()),
            JEV_MODEL,
            Some(var("OPENROUTER_API_KEY")?),
            JEV_RUNNER_UP,
        ),
    };
    Some(Endpoint {
        url,
        model,
        key,
        timeout: Duration::from_millis(
            parsed("HOLSTERED_TIMEOUT_MS").unwrap_or(DEFAULT_TIMEOUT_MS),
        ),
        pick_threshold: parsed("HOLSTERED_PICK_THRESHOLD"),
        runner_up_threshold: parsed("HOLSTERED_RUNNER_UP_THRESHOLD").unwrap_or(runner_up_threshold),
    })
}

fn parsed<T: std::str::FromStr>(name: &str) -> Option<T> {
    var(name).and_then(|v| v.trim().parse().ok())
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
