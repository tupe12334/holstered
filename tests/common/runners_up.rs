//! Jev and Kev answering with a pick and a runner-up at 0.25.

use super::{backend::jev_and_kev, claude_prompt, mock_jev, run::exec};
use mockito::Server;
use serde_json::json;

fn runner_up_answer() -> String {
    json!({"answers": {"skill": {"type": "choice", "choice": "github-pr-merge",
        "probabilities": {"github-pr-merge": 0.7, "github-pr-review": 0.25, "none": 0.05}}}})
    .to_string()
}

pub fn jev_then_kev_contexts(runner_up_threshold: Option<&str>) -> Vec<String> {
    (0..2)
        .map(|backend| {
            let mut server = Server::new();
            let mock = mock_jev(&mut server, 200, runner_up_answer(), 1);
            let mut env = jev_and_kev(&server)[backend].clone();
            env.extend(
                runner_up_threshold.map(|t| ("HOLSTERED_RUNNER_UP_THRESHOLD", t.to_owned())),
            );
            let out = exec(&claude_prompt(), &env);
            mock.assert();
            out["hookSpecificOutput"]["additionalContext"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect()
}
