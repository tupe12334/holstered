use super::*;
use serde_json::json;

fn skill(name: &str) -> Skill {
    Skill {
        name: name.into(),
        description: String::new(),
        path: Default::default(),
    }
}

fn answer(choice: &str, probabilities: Value) -> Value {
    json!({"answers": {"skill": {"choice": choice, "probabilities": probabilities}}})
}

#[test]
fn keeps_skills_at_or_over_the_threshold_best_first() {
    let skills = [skill("merge"), skill("rebase"), skill("ci"), skill("docs")];
    let pool: Vec<_> = skills.iter().collect();
    let probs = json!({"merge": 0.5, "ci": 0.2, "rebase": 0.25, "docs": 0.05, "none": 0.3});
    let got = picks(&answer("merge", probs), &pool, 0.2).unwrap();
    assert_eq!(got, ["merge", "rebase", "ci"]);
}

#[test]
fn caps_at_max_skills_and_drops_unoffered_names() {
    let skills = [skill("a"), skill("b"), skill("c"), skill("d")];
    let pool: Vec<_> = skills.iter().collect();
    let probs = json!({"a": 0.3, "b": 0.3, "c": 0.2, "d": 0.2, "ghost": 0.9});
    assert_eq!(picks(&answer("a", probs), &pool, 0.1).unwrap().len(), 3);
}

#[test]
fn none_unknown_or_missing_probabilities() {
    let skills = [skill("merge"), skill("rebase")];
    let pool: Vec<_> = skills.iter().collect();
    let probs = json!({"none": 0.6, "merge": 0.4});
    assert!(picks(&answer("none", probs), &pool, 0.2)
        .unwrap()
        .is_empty());
    assert!(picks(&answer("ghost", json!({})), &pool, 0.2)
        .unwrap()
        .is_empty());
    let bare = json!({"answers": {"skill": {"choice": "merge"}}});
    assert_eq!(picks(&bare, &pool, 0.2).unwrap(), ["merge"]);
    assert!(picks(&json!({}), &pool, 0.2).is_err());
}
