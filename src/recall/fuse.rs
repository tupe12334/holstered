//! Reciprocal rank fusion: each ranking adds 1 / (K + rank) to a skill, so a
//! skill near the top of either ranking makes the shortlist, and one high in
//! both leads it.

use crate::skills::Skill;

/// The usual constant; it damps the gap between the first few ranks.
const K: f64 = 60.0;

pub fn reciprocal_rank<'a>(rankings: &[Vec<&'a Skill>], limit: usize) -> Vec<&'a Skill> {
    let mut scores: Vec<(&'a Skill, f64)> = Vec::new();
    for ranking in rankings {
        for (rank, &skill) in ranking.iter().enumerate() {
            let add = 1.0 / (K + rank as f64);
            match scores.iter_mut().find(|(s, _)| std::ptr::eq(*s, skill)) {
                Some((_, score)) => *score += add,
                None => scores.push((skill, add)),
            }
        }
    }
    // Stable sort: ties keep the earlier ranking's order.
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    scores.into_iter().take(limit).map(|(s, _)| s).collect()
}

#[cfg(test)]
#[path = "fuse_tests.rs"]
mod tests;
