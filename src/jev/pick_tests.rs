use super::*;
use serde_json::json;

fn skill(name: &str) -> Skill {
    Skill {
        name: name.into(),
        description: String::new(),
        path: Default::default(),
    }
}

#[test]
fn offered_choice_is_the_pick_whatever_its_probability() {
    let skills = [skill("merge")];
    let pool: Vec<_> = skills.iter().collect();
    let answer =
        json!({"answers": {"skill": {"choice": "merge", "probabilities": {"merge": 0.05}}}});
    assert_eq!(pick(&answer, &pool).unwrap(), Some("merge"));
}

#[test]
fn none_or_unoffered_choice_is_no_pick() {
    let skills = [skill("merge")];
    let pool: Vec<_> = skills.iter().collect();
    for choice in ["none", "ghost"] {
        let answer = json!({"answers": {"skill": {"choice": choice}}});
        assert_eq!(pick(&answer, &pool).unwrap(), None);
    }
    assert!(pick(&json!({}), &pool).is_err());
}
