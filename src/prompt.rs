//! Clips a long prompt to what the router reads.
//!
//! Pasted logs or files usually come first and the request last ("<trace>
//! fix this"), so keeping only the head would hide the task. Keep both ends.

const HEAD: usize = 500;
const TAIL: usize = 1500;

/// The whole prompt when it fits, else its first `HEAD` and last `TAIL` chars.
pub fn clip(prompt: &str) -> String {
    let chars: Vec<char> = prompt.trim().chars().collect();
    if chars.len() <= HEAD + TAIL {
        return chars.into_iter().collect();
    }
    let head: String = chars[..HEAD].iter().collect();
    let tail: String = chars[chars.len() - TAIL..].iter().collect();
    format!("{head}\n…\n{tail}")
}

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod tests;
