//! Picks the skill for a prompt event: BM25 shortlist, then Jev decides.

use crate::{bm25, inject, jev, skills};
use polyhook::{HookEvent, HookEventEvent};

const POOL: usize = 20;
const PROMPT_LIMIT: usize = 2000;

/// The context to inject, or `None` to leave the prompt untouched.
pub fn select(event: &HookEvent) -> Option<String> {
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
    inject::context(pool.iter().find(|s| s.name == name)?)
}
