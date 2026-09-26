//! Where the decision goes: local Kev when configured, else Jev on OpenRouter.

use super::{DEFAULT_URL, JEV_MODEL, KEV_MODEL};
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 8000;

/// A decisions endpoint, the model it serves, its key if it takes one, and
/// how long to wait for its answer (`HOLSTERED_TIMEOUT_MS`, default 8s).
pub struct Endpoint {
    pub url: String,
    pub model: &'static str,
    pub key: Option<String>,
    pub timeout: Duration,
}

/// `HOLSTERED_KEV_URL` selects a local Kev server, which needs no key.
/// Otherwise Jev needs `OPENROUTER_API_KEY`; `None` when it is unset.
pub fn endpoint() -> Option<Endpoint> {
    let timeout = Duration::from_millis(
        var("HOLSTERED_TIMEOUT_MS")
            .and_then(|ms| ms.trim().parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_MS),
    );
    if let Some(url) = var("HOLSTERED_KEV_URL") {
        return Some(Endpoint {
            url,
            model: KEV_MODEL,
            key: None,
            timeout,
        });
    }
    let key = var("OPENROUTER_API_KEY")?;
    let url = var("HOLSTERED_JEV_URL").unwrap_or_else(|| DEFAULT_URL.into());
    Some(Endpoint {
        url,
        model: JEV_MODEL,
        key: Some(key),
        timeout,
    })
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
