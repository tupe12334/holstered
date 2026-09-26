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
fn pick_threshold_rejects_a_weak_pick_only_when_set() {
    let skills = [skill("merge")];
    let pool: Vec<_> = skills.iter().collect();
    let weak = json!({"answers": {"skill": {"choice": "merge", "probabilities": {"merge": 0.05}}}});
    assert_eq!(pick(&weak, &pool, None).unwrap(), Some("merge"));
    assert_eq!(pick(&weak, &pool, Some(0.1)).unwrap(), None);
    let bare = json!({"answers": {"skill": {"choice": "merge"}}});
    assert_eq!(pick(&bare, &pool, Some(0.9)).unwrap(), Some("merge"));
}

#[test]
fn none_or_unoffered_choice_is_no_pick() {
    let skills = [skill("merge")];
    let pool: Vec<_> = skills.iter().collect();
    for choice in ["none", "ghost"] {
        let answer = json!({"answers": {"skill": {"choice": choice}}});
        assert_eq!(pick(&answer, &pool, None).unwrap(), None);
    }
    assert!(pick(&json!({}), &pool, None).is_err());
}
