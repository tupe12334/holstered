//! Read holstered's decisions request: the prompt and the offered shortlist.

use serde_json::Value;

/// The prompt, and the offered skill names sorted, without the `none` option.
pub fn parse(body: &Value) -> (String, Vec<String>) {
    let prompt = body["state"]["user_prompt"].as_str().unwrap().to_owned();
    let mut offered: Vec<String> = body["questions"]["skill"]["criteria"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| *k != "none")
        .cloned()
        .collect();
    offered.sort();
    (prompt, offered)
}
