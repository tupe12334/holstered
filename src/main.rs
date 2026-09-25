//! holstered: a prompt hook that hands the agent the one skill it needs.
//!
//! On each user prompt it shortlists skills with BM25, asks the Jev decision
//! model to pick one (or none), and injects that skill's SKILL.md into the
//! model's context through polyhook, so one binary serves every agent.
//! Any failure, missing key, or `none` answer leaves the prompt untouched.

mod bm25;
mod jev;
mod skills;

use polyhook::{HookEvent, HookEventEvent, HookResponse};

const POOL: usize = 20;
const PROMPT_LIMIT: usize = 2000;
// Hook output past ~10KB is truncated by some agents; cap the injected body.
const BODY_LIMIT: usize = 8000;

fn main() {
    // A prompt hook must never block the user: unreadable input means no-op.
    let Ok(event) = polyhook::read() else {
        return;
    };
    let response = select(&event).map_or_else(HookResponse::approve, |c| HookResponse::context(&c));
    let _ = polyhook::respond(&response);
}

fn select(event: &HookEvent) -> Option<String> {
    if event.event != HookEventEvent::PromptSubmit {
        return None;
    }
    let prompt: String = event
        .prompt
        .as_deref()?
        .trim()
        .chars()
        .take(PROMPT_LIMIT)
        .collect();
    let key = std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())?;
    if prompt.is_empty() {
        return None;
    }

    let skills = skills::discover(&skills::dirs());
    let pool: Vec<_> = bm25::rank(&prompt, &skills, POOL)
        .into_iter()
        .filter(|s| s.name != jev::NONE)
        .collect();
    if pool.is_empty() {
        return None;
    }
    let url = std::env::var("HOLSTERED_JEV_URL").unwrap_or_else(|_| jev::DEFAULT_URL.into());
    let name = match jev::choose(&url, &key, &prompt, &pool) {
        Ok(name) => name?,
        Err(e) => {
            eprintln!("holstered: {e}");
            return None;
        }
    };
    let skill = pool.iter().find(|s| s.name == name)?;
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
        "SKILL SELECTED by holstered: {name} ({}). Follow it for this task.\n---\n{body}",
        skill.path.display()
    ))
}
