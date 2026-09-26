//! Where the decision goes: local Kev when configured, else Jev on OpenRouter.

use super::DEFAULT_URL;

/// A decisions endpoint and the key to send it, if it takes one.
pub struct Endpoint {
    pub url: String,
    pub key: Option<String>,
}

/// `HOLSTERED_KEV_URL` selects a local Kev server, which needs no key.
/// Otherwise Jev needs `OPENROUTER_API_KEY`; `None` when it is unset.
pub fn endpoint() -> Option<Endpoint> {
    if let Some(url) = var("HOLSTERED_KEV_URL") {
        return Some(Endpoint { url, key: None });
    }
    let key = var("OPENROUTER_API_KEY")?;
    let url = var("HOLSTERED_JEV_URL").unwrap_or_else(|| DEFAULT_URL.into());
    Some(Endpoint {
        url,
        key: Some(key),
    })
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
