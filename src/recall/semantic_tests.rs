use super::*;

fn skill(name: &str, description: &str) -> Skill {
    Skill {
        name: name.into(),
        description: description.into(),
        path: Default::default(),
    }
}

#[test]
fn finds_a_skill_that_shares_no_word_with_the_prompt() {
    let skills = [
        skill("github-pr-merge", "Merge pull requests safely."),
        skill(
            "recipe-planning",
            "Plan meals and recipes from ingredients on hand.",
        ),
        skill("docker-image-build", "Write Dockerfiles and build images."),
    ];
    let got = rank("what can I cook with eggs, rice and spinach?", &skills, 3);
    assert_eq!(got[0].name, "recipe-planning");
    assert_eq!(got.len(), 3);
    assert_eq!(rank("anything", &skills, 1).len(), 1);
}
