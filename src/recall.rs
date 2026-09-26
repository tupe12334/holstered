//! The shortlist the decision model chooses from: a keyword ranking (BM25)
//! and a meaning ranking (static embeddings), fused. Keywords alone miss a
//! prompt that shares no word with the skill ("what can I cook with eggs?"
//! vs "Plan meals and recipes").

mod bm25;
mod fuse;
mod semantic;

use crate::skills::Skill;

/// The `limit` skills most likely to fit `query`, best first.
pub fn shortlist<'a>(query: &str, skills: &'a [Skill], limit: usize) -> Vec<&'a Skill> {
    let by_keyword = bm25::rank(query, skills, limit);
    let by_meaning = semantic::rank(query, skills, limit);
    fuse::reciprocal_rank(&[by_keyword, by_meaning], limit)
}

/// What both rankings read for a skill: its name as words, then its description.
fn document(skill: &Skill) -> String {
    format!("{} {}", skill.name.replace('-', " "), skill.description)
}
