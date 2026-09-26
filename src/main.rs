//! holstered: a prompt hook that hands the agent the skills it needs.
//!
//! On each user prompt it shortlists skills with BM25, asks the Jev decision
//! model to pick one (or none) plus, when a probability threshold is set, any
//! runners-up over it, and injects those SKILL.md files into the model's context through polyhook, so one binary serves every agent.
//! Any failure, missing key, or `none` answer leaves the prompt untouched.

mod bm25;
mod inject;
mod jev;
mod select;
mod skills;

use polyhook::HookResponse;

fn main() {
    // A prompt hook must never block the user: unreadable input means no-op.
    let Ok(event) = polyhook::read() else {
        return;
    };
    let response =
        select::select(&event).map_or_else(HookResponse::approve, |c| HookResponse::context(&c));
    let _ = polyhook::respond(&response);
}
