//! Skill discovery: every `SKILL.md` under the configured skill directories.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
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

// Hermes nests skills under category folders; four levels covers it and
// bounds the walk if a symlink loops.
const MAX_DEPTH: usize = 4;

/// Skills with a description, first occurrence winning. The same skill is
/// often mirrored into several agents' folders under different names, so a
/// repeated description counts as a duplicate too.
pub fn discover(dirs: &[PathBuf]) -> Vec<Skill> {
    let mut files = Vec::new();
    for dir in dirs {
        walk(dir, 0, &mut files);
    }
    let (mut names, mut descriptions) = (HashSet::new(), HashSet::new());
    files
        .into_iter()
        .filter_map(|path| load(&path))
        .filter(|s| names.insert(s.name.clone()) && descriptions.insert(s.description.clone()))
        .collect()
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        let hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            if depth < MAX_DEPTH {
                walk(&path, depth + 1, out);
            }
        } else if path.file_name().is_some_and(|n| n == "SKILL.md") {
            out.push(path);
        }
    }
}

fn load(path: &Path) -> Option<Skill> {
    let text = std::fs::read_to_string(path).ok()?;
    let block = frontmatter(&text)?;
    let description = field(block, "description").filter(|d| !d.is_empty())?;
    let name = field(block, "name")
        .filter(|n| !n.is_empty())
        .or_else(|| path.parent()?.file_name()?.to_str().map(str::to_owned))?;
    Some(Skill {
        name,
        description,
        path: path.to_owned(),
    })
}

fn frontmatter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---")?.trim_start_matches(['\r', ' ']);
    let rest = rest.strip_prefix('\n')?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// A top-level scalar: plain, quoted, or a `>`/`|` block.
fn field(block: &str, key: &str) -> Option<String> {
    let mut lines = block.lines();
    let prefix = format!("{key}:");
    let value = lines.by_ref().find_map(|l| l.strip_prefix(&prefix))?.trim();
    let value = if matches!(value, "" | ">" | ">-" | "|" | "|-") {
        lines
            .take_while(|l| l.trim().is_empty() || l.starts_with([' ', '\t']))
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        value.to_owned()
    };
    let unquoted = ['"', '\'']
        .iter()
        .find_map(|q| value.strip_prefix(*q)?.strip_suffix(*q))
        .unwrap_or(&value);
    Some(unquoted.split_whitespace().collect::<Vec<_>>().join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn parses_plain_quoted_and_block_descriptions() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "a/SKILL.md",
            "---\nname: alpha\ndescription: Plain one.\n---\nbody",
        );
        write(
            dir.path(),
            "b/SKILL.md",
            "---\ndescription: \"Quoted: two.\"\n---\n",
        );
        write(
            dir.path(),
            "c/SKILL.md",
            "---\nname: gamma\ndescription: >-\n  Folded\n  three.\nversion: 1\n---\n",
        );
        let skills = discover(&[dir.path().to_owned()]);
        let got: Vec<_> = skills
            .iter()
            .map(|s| (s.name.as_str(), s.description.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                ("alpha", "Plain one."),
                ("b", "Quoted: two."),
                ("gamma", "Folded three.")
            ]
        );
    }

    #[test]
    fn walks_nested_dirs_skips_hidden_and_undescribed() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "cat/deep/SKILL.md",
            "---\nname: deep\ndescription: Nested.\n---\n",
        );
        write(
            dir.path(),
            ".hidden/SKILL.md",
            "---\nname: hidden\ndescription: No.\n---\n",
        );
        write(dir.path(), "bare/SKILL.md", "no frontmatter");
        let names: Vec<_> = discover(&[dir.path().to_owned()])
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["deep"]);
    }

    #[test]
    fn first_occurrence_wins_across_dirs() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(
            a.path(),
            "x/SKILL.md",
            "---\nname: x\ndescription: Same text.\n---\n",
        );
        write(
            b.path(),
            "x/SKILL.md",
            "---\nname: x\ndescription: Other text.\n---\n",
        );
        write(
            b.path(),
            "cat/y/SKILL.md",
            "---\nname: y\ndescription: Same text.\n---\n",
        );
        let skills = discover(&[a.path().to_owned(), b.path().to_owned()]);
        assert_eq!(skills.len(), 1);
        assert!(skills[0].path.starts_with(a.path()));
    }
}
