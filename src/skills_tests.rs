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
