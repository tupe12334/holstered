//! Where skills live: each served agent's skill folder under `$HOME`.

use std::path::PathBuf;

/// Skill roots of the agents holstered serves, or `HOLSTERED_SKILLS_DIRS`
/// (a PATH-style list) when set.
pub fn dirs() -> Vec<PathBuf> {
    if let Some(custom) = std::env::var_os("HOLSTERED_SKILLS_DIRS") {
        return std::env::split_paths(&custom).collect();
    }
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    [
        ".claude/skills",
        ".codex/skills",
        ".agents/skills",
        ".gemini/skills",
        ".hermes/skills",
    ]
    .iter()
    .map(|d| home.join(d))
    .collect()
}
