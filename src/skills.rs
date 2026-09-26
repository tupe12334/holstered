//! Skill discovery: every `SKILL.md` under the configured skill directories.

mod load;
mod roots;

pub use roots::dirs;

use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

// Hermes nests skills under category folders; this depth covers it and bounds
// the walk if a followed symlink loops.
const MAX_DEPTH: usize = 5;

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
        .filter_map(|entry| load::load(entry.path()))
        .filter(|s| names.insert(s.name.clone()) && descriptions.insert(s.description.clone()))
        .collect()
}

#[cfg(test)]
#[path = "skills_tests.rs"]
mod tests;
