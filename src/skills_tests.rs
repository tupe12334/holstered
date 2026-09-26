use super::*;
use std::path::Path;

#[path = "skills_walk_tests.rs"]
mod walk;

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
