use super::SKILL_BODY;

/// Three skills: two GitHub PR skills, one unrelated.
pub fn skills_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, desc, body) in [
        (
            "github-pr-merge",
            "Merge GitHub pull requests safely.",
            SKILL_BODY,
        ),
        (
            "github-pr-review",
            "Review a GitHub pull request diff.",
            "Review body.",
        ),
        ("cooking", "Bake bread.", "Knead."),
    ] {
        let path = dir.path().join(name);
        std::fs::create_dir(&path).unwrap();
        std::fs::write(
            path.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {desc}\n---\n{body}\n"),
        )
        .unwrap();
    }
    dir
}
