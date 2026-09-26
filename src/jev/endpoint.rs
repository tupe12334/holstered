//! Where the decision goes: local Kev when configured, else Jev on OpenRouter.

use super::{DEFAULT_URL, JEV_MODEL, KEV_MODEL};
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 8000;
// Jev put a two-task prompt's second skill at 0.21; one-task prompts sit near 1.
const DEFAULT_THRESHOLD: f64 = 0.2;

/// A decisions endpoint, its model and key, how long to wait for its answer
/// (`HOLSTERED_TIMEOUT_MS`, default 8s), and the probability a runner-up
/// skill needs to be injected too (`HOLSTERED_THRESHOLD`, default 0.2).
pub struct Endpoint {
    pub url: String,
    pub model: &'static str,
    pub key: Option<String>,
    pub timeout: Duration,
    pub threshold: f64,
}

/// `HOLSTERED_KEV_URL` selects a local Kev server, which needs no key.
/// Otherwise Jev needs `OPENROUTER_API_KEY`; `None` when it is unset.
pub fn endpoint() -> Option<Endpoint> {
    let (url, model, key) = match var("HOLSTERED_KEV_URL") {
        Some(url) => (url, KEV_MODEL, None),
        None => (
            var("HOLSTERED_JEV_URL").unwrap_or_else(|| DEFAULT_URL.into()),
            JEV_MODEL,
            Some(var("OPENROUTER_API_KEY")?),
        ),
    };
    Some(Endpoint {
        url,
        model,
        key,
        timeout: Duration::from_millis(num("HOLSTERED_TIMEOUT_MS", DEFAULT_TIMEOUT_MS)),
        threshold: num("HOLSTERED_THRESHOLD", DEFAULT_THRESHOLD),
    })
}

fn num<T: std::str::FromStr>(name: &str, default: T) -> T {
    var(name)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
