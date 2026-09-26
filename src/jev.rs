//! The Jev decision model, called through OpenRouter's Decisions API.
//!
//! Request and response shapes follow the typed-decision-models skill
//! (`references/jev-system-one-api.md`).

mod request;

use crate::skills::Skill;
use serde_json::Value;
use std::time::Duration;

pub const DEFAULT_URL: &str = "https://openrouter.ai/api/alpha/decisions";
pub const MODEL: &str = "~typesafe/jev-latest";
pub const NONE: &str = "none";
const TIMEOUT: Duration = Duration::from_secs(8);

/// Ask Jev which of `pool` fits `prompt`. `Ok(None)` when it answers `none`
/// or names a skill it was not offered.
pub fn choose(
    url: &str,
    key: &str,
    prompt: &str,
    pool: &[&Skill],
) -> Result<Option<String>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build()
        .into();
    // Errors carry the status or transport failure only, never request headers,
    // so the key cannot leak through them.
    let answer: Value = agent
        .post(url)
        .header("authorization", &format!("Bearer {key}"))
        .send_json(request::body(prompt, pool))
        .map_err(|e| format!("jev request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("jev response unreadable: {e}"))?;

    let choice = answer["answers"]["skill"]["choice"]
        .as_str()
        .ok_or("jev response has no answers.skill.choice")?;
    Ok(pool
        .iter()
        .any(|s| s.name == choice)
        .then(|| choice.to_owned()))
}
