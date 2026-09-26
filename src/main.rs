//! holstered: a prompt hook that hands the agent the skills it needs.
//!
//! On each user prompt it shortlists skills with BM25, asks the Jev decision
//! model to pick one (or none) plus, when a runner-up threshold is set, any
//! runners-up over it, and injects those SKILL.md files into the model's context through polyhook, so one binary serves every agent.
//! Any failure, missing key, or `none` answer leaves the prompt untouched.

mod bm25;
mod inject;
mod jev;
mod select;
mod skills;

use clap::Parser;
use polyhook::HookResponse;

// Only flags are parsed; the hook payload always arrives on stdin.
#[derive(Parser)]
#[command(
    version,
    about,
    disable_version_flag = true,
    after_help = "Run with no flags as a prompt hook: reads the hook event JSON on stdin."
)]
struct Cli {
    /// Print version
    #[arg(short = 'v', long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    Cli::parse();
    // A prompt hook must never block the user: unreadable input means no-op.
    let Ok(event) = polyhook::read() else {
        return;
    };
    let response =
        select::select(&event).map_or_else(HookResponse::approve, |c| HookResponse::context(&c));
    let _ = polyhook::respond(&response);
}
