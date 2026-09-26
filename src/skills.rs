//! Skill discovery: every `SKILL.md` under the configured skill directories.

use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

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

// Hermes nests skills under category folders; this depth covers it and bounds
// the walk if a followed symlink loops.
const MAX_DEPTH: usize = 5;

#[derive(Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
}

/// Skills with a description, first occurrence winning. The same skill is
/// often mirrored into several agents' folders under different names, so a
/// repeated description counts as a duplicate too.
pub fn discover(dirs: &[PathBuf]) -> Vec<Skill> {
    let (mut names, mut descriptions) = (HashSet::new(), HashSet::new());
    dirs.iter()
        .flat_map(|dir| {
            WalkDir::new(dir)
                .follow_links(true)
                .max_depth(MAX_DEPTH)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|e| {
                    e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.')
                })
                .flatten()
                .filter(|e| e.file_type().is_file() && e.file_name() == "SKILL.md")
        })
        .filter_map(|entry| load(entry.path()))
        .filter(|s| names.insert(s.name.clone()) && descriptions.insert(s.description.clone()))
        .collect()
}

fn load(path: &Path) -> Option<Skill> {
    let text = std::fs::read_to_string(path).ok()?;
    let meta: Frontmatter = Matter::<YAML>::new().parse(&text).ok()?.data?;
    let description = meta
        .description
        .map(|d| d.split_whitespace().collect::<Vec<_>>().join(" "));
    let description = description.filter(|d| !d.is_empty())?;
    let name = meta
        .name
        .filter(|n| !n.is_empty())
        .or_else(|| path.parent()?.file_name()?.to_str().map(str::to_owned))?;
    Some(Skill {
        name,
        description,
        path: path.to_owned(),
    })
}

#[cfg(test)]
#[path = "skills_tests.rs"]
mod tests;
