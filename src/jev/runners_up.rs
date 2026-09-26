//! Runners-up: other offered skills whose probability reaches the threshold.

use super::pick::offered;
use crate::skills::Skill;
use serde_json::Value;

/// Beside the pick; more SKILL.md bodies crowd the agent's context.
const MAX_RUNNERS_UP: usize = 2;

/// Best first, at most `MAX_RUNNERS_UP`, never `pick` itself.
pub fn runners_up<'a>(
    answer: &'a Value,
    pool: &[&Skill],
    pick: &str,
    threshold: f64,
) -> Vec<&'a str> {
    let mut ranked: Vec<(&str, f64)> = answer["answers"]["skill"]["probabilities"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(name, p)| Some((name.as_str(), p.as_f64()?)))
        .filter(|&(name, p)| name != pick && p >= threshold && offered(name, pool))
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked
        .into_iter()
        .take(MAX_RUNNERS_UP)
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
#[path = "runners_up_tests.rs"]
mod tests;
