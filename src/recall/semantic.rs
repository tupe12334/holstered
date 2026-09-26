//! Meaning recall: ranks skills by embedding similarity, so a prompt finds a
//! skill it shares no word with. The model (potion-base-4M, int8) is compiled
//! in, so recall stays offline and needs no download.

use super::document;
use crate::skills::Skill;
use model2vec_rs::model::StaticModel;

const MODEL: &[u8] = include_bytes!("../../models/potion-base-4M-int8/model.safetensors");
const TOKENIZER: &[u8] = include_bytes!("../../models/potion-base-4M-int8/tokenizer.json");
const CONFIG: &[u8] = include_bytes!("../../models/potion-base-4M-int8/config.json");

/// The `limit` skills closest in meaning to `query`, best first. Empty if the
/// model fails to load, leaving recall to BM25.
pub fn rank<'a>(query: &str, skills: &'a [Skill], limit: usize) -> Vec<&'a Skill> {
    let Ok(model) = StaticModel::from_bytes(TOKENIZER, MODEL, CONFIG, None) else {
        return Vec::new();
    };
    let texts: Vec<String> = std::iter::once(query.to_owned())
        .chain(skills.iter().map(document))
        .collect();
    let vectors = model.encode(&texts);
    let Some((query, docs)) = vectors.split_first() else {
        return Vec::new();
    };
    // The model normalizes its vectors, so the dot product is the cosine.
    let mut ranked: Vec<(usize, f32)> = docs
        .iter()
        .map(|d| d.iter().zip(query).map(|(a, b)| a * b).sum())
        .enumerate()
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked
        .into_iter()
        .take(limit)
        .map(|(i, _)| &skills[i])
        .collect()
}

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;
