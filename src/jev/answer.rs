//! Reads the skills to inject out of Jev's `choice` answer.

use super::NONE;
use crate::skills::Skill;
use serde_json::Value;

/// More than this many SKILL.md bodies crowd the agent's context.
const MAX_SKILLS: usize = 3;

/// The pick plus every other offered skill whose probability reaches
/// `threshold`, best first, at most `MAX_SKILLS`. Empty when Jev answers
/// `none` or names a skill it was not offered.
pub fn picks(answer: &Value, pool: &[&Skill], threshold: f64) -> Result<Vec<String>, String> {
    let skill = &answer["answers"]["skill"];
    let choice = skill["choice"]
        .as_str()
        .ok_or("jev response has no answers.skill.choice")?;
    let offered = |name: &str| name != NONE && pool.iter().any(|s| s.name == name);
    if !offered(choice) {
        return Ok(Vec::new());
    }
    let mut ranked: Vec<(&str, f64)> = skill["probabilities"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(name, p)| Some((name.as_str(), p.as_f64()?)))
        .filter(|&(name, p)| name != choice && p >= threshold && offered(name))
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    Ok(std::iter::once(choice)
        .chain(ranked.into_iter().map(|(name, _)| name))
        .take(MAX_SKILLS)
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
#[path = "answer_tests.rs"]
mod tests;
