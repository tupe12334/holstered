//! Score one case: was the right skill shortlisted, picked, or correctly
//! left out. A miss names its stage: retrieval (BM25 never offered the right
//! skill) or routing (it was offered and the model chose otherwise).

use super::data::{Baseline, Case};

pub fn tally(
    case: &Case,
    pick: Option<String>,
    offered: Option<Vec<String>>,
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
    let shortlisted = want
        .iter()
        .any(|w| offered.iter().flatten().any(|o| o == w));
    score.recalled += usize::from(shortlisted);
    if pick.as_ref().is_some_and(|s| want.contains(s)) {
        score.correct += 1;
    } else if shortlisted {
        misses.push(format!(
            "routing miss: {p:?} -> {pick:?} (want {want:?}, shortlisted)"
        ));
    } else {
        misses.push(format!(
            "retrieval miss: {p:?} (want {want:?}, not shortlisted)"
        ));
    }
}
