//! Jev's pick: the option it scored highest, `none` included.

use super::NONE;
use crate::skills::Skill;
use serde_json::Value;

/// The skill Jev chose, or `None` when it chose `none` or a skill it was not offered.
pub fn pick<'a>(answer: &'a Value, pool: &[&Skill]) -> Result<Option<&'a str>, String> {
    let choice = answer["answers"]["skill"]["choice"]
        .as_str()
        .ok_or("jev response has no answers.skill.choice")?;
    Ok(offered(choice, pool).then_some(choice))
}

pub fn offered(name: &str, pool: &[&Skill]) -> bool {
    name != NONE && pool.iter().any(|s| s.name == name)
}

#[cfg(test)]
#[path = "pick_tests.rs"]
mod tests;
