//! The decision model's pick: the option it scored highest, `none` included.

use super::NONE;
use crate::skills::Skill;
use serde_json::Value;

/// The skill the model chose, or `None` when it chose `none`, a skill it was
/// not offered, or one it scored under `threshold`.
pub fn pick<'a>(
    answer: &'a Value,
    pool: &[&Skill],
    threshold: Option<f64>,
) -> Result<Option<&'a str>, String> {
    let skill = &answer["answers"]["skill"];
    let choice = skill["choice"]
        .as_str()
        .ok_or("jev response has no answers.skill.choice")?;
    // ponytail: an answer without probabilities passes the threshold, as before.
    let weak = threshold
        .zip(skill["probabilities"][choice].as_f64())
        .is_some_and(|(t, p)| p < t);
    Ok((offered(choice, pool) && !weak).then_some(choice))
}

pub fn offered(name: &str, pool: &[&Skill]) -> bool {
    name != NONE && pool.iter().any(|s| s.name == name)
}

#[cfg(test)]
#[path = "pick_tests.rs"]
mod tests;
