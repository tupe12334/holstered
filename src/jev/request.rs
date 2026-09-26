//! The Decisions API request: one `choice` question over the shortlist + `none`.

use super::NONE;
use crate::skills::Skill;
use serde_json::{json, Map, Value};

pub fn body(model: &str, prompt: &str, pool: &[&Skill]) -> Value {
    let mut criteria: Map<String, Value> = pool
        .iter()
        .map(|s| {
            let text = format!(
                "The user's prompt is a task this skill covers: {}",
                s.description
            );
            (s.name.clone(), Value::String(text))
        })
        .collect();
    criteria.insert(
        NONE.into(),
        "None of the other options covers the task in the user's prompt.".into(),
    );
    json!({
        "state": { "user_prompt": prompt },
        "model": model,
        "questions": { "skill": {
            "type": "choice",
            "instructions": "Pick the one skill the coding agent should load for this prompt, or none.",
            "criteria": criteria,
        }},
    })
}
