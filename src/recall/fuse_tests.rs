use super::*;

fn skill(name: &str) -> Skill {
    Skill {
        name: name.into(),
        description: String::new(),
        path: Default::default(),
    }
}

#[test]
fn a_skill_high_in_both_rankings_leads() {
    let [a, b, c] = [skill("a"), skill("b"), skill("c")];
    let got = reciprocal_rank(&[vec![&a, &b], vec![&b, &c]], 10);
    let names: Vec<_> = got.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["b", "a", "c"]);
}

#[test]
fn keeps_skills_found_by_one_ranking_and_respects_limit() {
    let [a, b, c] = [skill("a"), skill("b"), skill("c")];
    assert_eq!(reciprocal_rank(&[vec![], vec![&c]], 10)[0].name, "c");
    assert_eq!(reciprocal_rank(&[vec![&a, &b], vec![&c]], 2).len(), 2);
}
