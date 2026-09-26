//! How sure the model was: the gap between `none` and the best skill on each
//! no-skill case, and what a pick threshold would have scored. Informational;
//! it calibrates `HOLSTERED_PICK_THRESHOLD` and never fails the test.

use super::clip::clip;
use super::data::{cases, Cassette};

const THRESHOLDS: [f64; 7] = [0.0, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7];

pub fn report(tape: &Cassette) -> Vec<String> {
    if tape.values().all(|d| d.probabilities.is_empty()) {
        return vec!["no probabilities recorded".into()];
    }
    // A prompt that never reached the model has no decision: it stays silent.
    let cases: Vec<_> = cases()
        .into_iter()
        .map(|c| (tape.get(&clip(&c.prompt)), c))
        .collect();
    let mut lines: Vec<String> = cases
        .iter()
        .filter(|(_, c)| c.expect.is_empty())
        .filter_map(|(d, c)| {
            let d = (*d)?;
            let (skill, p) = d.best_skill()?;
            let prompt: String = c.prompt.chars().take(50).collect();
            Some(format!(
                "none {:.2} vs {skill} {p:.2}: {prompt:?}",
                d.p("none")
            ))
        })
        .collect();
    for t in THRESHOLDS {
        let (mut correct, mut abstained) = (0, 0);
        for (d, c) in &cases {
            let picked = d.filter(|d| d.choice != "none" && d.p(&d.choice) >= t);
            let picked = picked.map(|d| &d.choice);
            correct += usize::from(picked.is_some_and(|s| c.expect.contains(s)));
            abstained += usize::from(c.expect.is_empty() && picked.is_none());
        }
        lines.push(format!(
            "threshold {t:.1}: correct {correct}, abstained {abstained}"
        ));
    }
    lines
}
