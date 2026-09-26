//! The Jev decision model, called through OpenRouter's Decisions API, or
//! Kev, an open-weights model serving the same API locally.
//!
//! Request and response shapes follow the typed-decision-models skill
//! (`references/jev-system-one-api.md`).

mod ask;
mod endpoint;
mod pick;
mod request;
mod runners_up;

pub use endpoint::{endpoint, Endpoint};

use crate::skills::Skill;

pub const DEFAULT_URL: &str = "https://openrouter.ai/api/alpha/decisions";
pub const JEV_MODEL: &str = "~typesafe/jev-latest";
pub const KEV_MODEL: &str = "kev-latest";
pub const NONE: &str = "none";

/// Ask Jev or Kev which of `pool` fit `prompt`: its pick, then its runners-up when
/// a runner-up threshold is set. Empty without a pick.
pub fn choose(at: &Endpoint, prompt: &str, pool: &[&Skill]) -> Result<Vec<String>, String> {
    let answer = ask::ask(at, prompt, pool)?;
    let Some(pick) = pick::pick(&answer, pool, at.pick_threshold)? else {
        return Ok(Vec::new());
    };
    let runners_up = at
        .runner_up_threshold
        .map(|t| runners_up::runners_up(&answer, pool, pick, t))
        .unwrap_or_default();
    Ok(std::iter::once(pick)
        .chain(runners_up)
        .map(str::to_owned)
        .collect())
}
