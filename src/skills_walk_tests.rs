//! Directory walking: nesting, hidden folders and cross-root dedup.

use super::super::discover;
use super::write;

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
