use super::*;
use serde_json::json;

fn skill(name: &str) -> Skill {
    Skill {
        name: name.into(),
        description: String::new(),
        path: Default::default(),
    }
}

fn answer(probabilities: &Value) -> Value {
    json!({"answers": {"skill": {"choice": "merge", "probabilities": probabilities}}})
}

#[test]
fn at_or_over_threshold_best_first_without_the_pick() {
    let skills = [skill("merge"), skill("rebase"), skill("ci"), skill("docs")];
    let pool: Vec<_> = skills.iter().collect();
    let probs = json!({"merge": 0.5, "ci": 0.2, "rebase": 0.25, "docs": 0.05, "none": 0.3});
    assert_eq!(
        runners_up(&answer(&probs), &pool, "merge", 0.2),
        ["rebase", "ci"]
    );
}

#[test]
fn capped_and_offered_only() {
    let skills = [skill("merge"), skill("a"), skill("b"), skill("c")];
    let pool: Vec<_> = skills.iter().collect();
    let probs = json!({"a": 0.3, "b": 0.3, "c": 0.2, "ghost": 0.9});
    assert_eq!(
        runners_up(&answer(&probs), &pool, "merge", 0.1).len(),
        MAX_RUNNERS_UP
    );
}

#[test]
fn missing_probabilities_mean_no_runners_up() {
    let skills = [skill("merge"), skill("rebase")];
    let pool: Vec<_> = skills.iter().collect();
    let bare = json!({"answers": {"skill": {"choice": "merge"}}});
    assert!(runners_up(&bare, &pool, "merge", 0.0).is_empty());
}
