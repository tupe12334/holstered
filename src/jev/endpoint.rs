//! Where the decision goes: local Kev when configured, else Jev on OpenRouter.

use super::{DEFAULT_URL, JEV_MODEL, KEV_MODEL};

/// A decisions endpoint, the model it serves, and its key if it takes one.
pub struct Endpoint {
    pub url: String,
    pub model: &'static str,
    pub key: Option<String>,
}

/// `HOLSTERED_KEV_URL` selects a local Kev server, which needs no key.
/// Otherwise Jev needs `OPENROUTER_API_KEY`; `None` when it is unset.
pub fn endpoint() -> Option<Endpoint> {
    if let Some(url) = var("HOLSTERED_KEV_URL") {
        return Some(Endpoint {
            url,
            model: KEV_MODEL,
            key: None,
        });
    }
    let key = var("OPENROUTER_API_KEY")?;
    let url = var("HOLSTERED_JEV_URL").unwrap_or_else(|| DEFAULT_URL.into());
    Some(Endpoint {
        url,
        model: JEV_MODEL,
        key: Some(key),
    })
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}
