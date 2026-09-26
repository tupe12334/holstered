//! Parses one `SKILL.md`: YAML frontmatter `name` and `description`.

use super::Skill;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
}

/// `None` without a non-empty description; the name falls back to the folder.
pub fn load(path: &Path) -> Option<Skill> {
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
