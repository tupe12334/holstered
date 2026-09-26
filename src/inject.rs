//! Renders the chosen skill as the context block handed to the agent.

use crate::skills::Skill;

// Hook output past ~10KB is truncated by some agents; cap the injected body.
const BODY_LIMIT: usize = 8000;

pub fn context(skill: &Skill) -> Option<String> {
    let mut body = std::fs::read_to_string(&skill.path).ok()?;
    if body.len() > BODY_LIMIT {
        let cut = (0..=BODY_LIMIT)
            .rev()
            .find(|&i| body.is_char_boundary(i))
            .unwrap_or(0);
        body.truncate(cut);
        body.push_str(&format!(
            "\n... (truncated; read {} for the rest)",
            skill.path.display()
        ));
    }
    Some(format!(
        "SKILL SELECTED by holstered: {} ({}). Follow it for this task.\n---\n{body}",
        skill.name,
        skill.path.display()
    ))
}
