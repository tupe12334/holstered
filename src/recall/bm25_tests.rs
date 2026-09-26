use super::*;

fn skill(name: &str, description: &str) -> Skill {
    Skill {
        name: name.into(),
        description: description.into(),
        path: Default::default(),
    }
}

#[test]
fn ranks_matching_skill_first_and_drops_non_matches() {
    let skills = [
        skill(
            "productivity-powerpoint",
            "Create and edit slide decks and presentations.",
        ),
        skill("github-pr-merge", "Merge pull requests safely."),
        skill("cooking", "Bake bread."),
    ];
    let got: Vec<_> = rank("merging the pull request", &skills, 20)
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(got, ["github-pr-merge"]);
    assert_eq!(
        rank("make a slide deck", &skills, 20)[0].name,
        "productivity-powerpoint"
    );
}

#[test]
fn respects_limit_and_handles_empty() {
    let skills = [
        skill("a", "merge"),
        skill("b", "merge"),
        skill("c", "merge"),
    ];
    assert_eq!(rank("merge", &skills, 2).len(), 2);
    assert!(rank("merge", &[], 5).is_empty());
    assert!(rank("the and", &skills, 5).is_empty());
}
