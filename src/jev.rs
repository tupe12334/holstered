//! The Jev decision model, called through OpenRouter's Decisions API, or
//! Kev, an open-weights model serving the same API locally.
//!
//! Request and response shapes follow the typed-decision-models skill
//! (`references/jev-system-one-api.md`).

mod answer;
mod endpoint;
mod request;

pub use endpoint::{endpoint, Endpoint};

use crate::skills::Skill;
use serde_json::Value;

pub const DEFAULT_URL: &str = "https://openrouter.ai/api/alpha/decisions";
pub const JEV_MODEL: &str = "~typesafe/jev-latest";
pub const KEV_MODEL: &str = "kev-latest";
pub const NONE: &str = "none";

/// Ask Jev which of `pool` fit `prompt`: its pick plus any runners-up at or
/// over the endpoint's threshold. Empty when it answers `none`.
pub fn choose(at: &Endpoint, prompt: &str, pool: &[&Skill]) -> Result<Vec<String>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(at.timeout))
        .build()
        .into();
    // Errors carry the status or transport failure only, never request headers,
    // so the key cannot leak through them.
    let mut req = agent.post(&at.url);
    if let Some(key) = &at.key {
        req = req.header("authorization", &format!("Bearer {key}"));
    }
    let answer: Value = req
        .send_json(request::body(at.model, prompt, pool))
        .map_err(|e| format!("jev request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("jev response unreadable: {e}"))?;

    answer::picks(&answer, pool, at.threshold)
}
