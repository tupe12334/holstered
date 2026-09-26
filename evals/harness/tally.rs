//! Score one case: was the right skill shortlisted, picked, or correctly
//! left out.

use super::data::{Baseline, Case};

pub fn tally(
    case: &Case,
    pick: Option<&str>,
    offered: Option<&[String]>,
    score: &mut Baseline,
    misses: &mut Vec<String>,
) {
    let (p, want) = (&case.prompt, &case.expect);
    if want.is_empty() {
        score.abstained += usize::from(pick.is_none());
        if pick.is_some() {
            misses.push(format!("should stay silent: {p:?} -> {pick:?}"));
        }
        return;
    }
    if want
        .iter()
        .any(|w| offered.into_iter().flatten().any(|o| o == w))
    {
        score.recalled += 1;
    } else {
        misses.push(format!("not shortlisted: {p:?} (want {want:?})"));
    }
    if pick.is_some_and(|s| want.iter().any(|w| w == s)) {
        score.correct += 1;
    } else {
        misses.push(format!("wrong pick: {p:?} -> {pick:?} (want {want:?})"));
    }
}
