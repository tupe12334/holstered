//! Picks the skills for a prompt event: keyword + meaning shortlist, then Jev decides.

use crate::{inject, jev, prompt, recall, skills};
use polyhook::{HookEvent, HookEventEvent};

const POOL: usize = 20;

/// The context to inject, or `None` to leave the prompt untouched.
pub fn select(event: &HookEvent) -> Option<String> {
    if event.event != HookEventEvent::PromptSubmit {
        return None;
    }
    let prompt = prompt::clip(event.prompt.as_deref()?);
    let at = jev::endpoint()?;
    if prompt.is_empty() {
        return None;
    }

    let skills = skills::discover(&skills::dirs());
    let pool: Vec<_> = recall::shortlist(&prompt, &skills, POOL)
        .into_iter()
        .filter(|s| s.name != jev::NONE)
        .collect();
    if pool.is_empty() {
        return None;
    }
    let names = jev::choose(&at, &prompt, &pool)
        .map_err(|e| eprintln!("holstered: {e}"))
        .ok()?;
    let picked: Vec<_> = names
        .iter()
        .filter_map(|n| pool.iter().find(|s| &s.name == n))
        .collect();
    let blocks: Vec<_> = picked
        .iter()
        .filter_map(|s| inject::context(s, picked.len()))
        .collect();
    (!blocks.is_empty()).then(|| blocks.join("\n\n"))
}
