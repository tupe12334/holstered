# Skill-selection eval

A fixed, labelled benchmark for holstered's pick, run by `cargo test` on every
PR. It is deterministic and offline: model decisions are replayed from
recordings, so a change to retrieval, the request, or the decision model shows
up as a failing test instead of a vague feeling.

| Path | What it holds |
|---|---|
| `skills/` | 30 mocked general-purpose skills (git, CI, containers, databases, docs, office work, travel, …), with deliberate near-neighbours such as PR review vs. PR merge |
| `cases.json` | Prompts with the skills that count as correct; `"expect": []` means the right answer is to stay silent |
| `cassettes/<model>.json` | For each prompt that reached the model: the shortlist it was offered, what it chose, and its probability for each option |
| `baseline.json` | Per model: `correct` picks, `abstained` on no-skill prompts, `recalled` (right skill was in the BM25 shortlist) |
| `eval.rs`, `harness/` | The test itself, registered in `Cargo.toml` as the `eval` test target |

Each miss is printed with its stage: a **retrieval miss** means BM25 never
shortlisted the right skill, so the fix is in search; a **routing miss** means
it was shortlisted and the model chose otherwise. Half of the no-skill prompts
are chit-chat that shares words with a skill ("thanks, the merge went
through fine"), so the shortlist is full of plausible but wrong skills.

## Confidence report

Every run also prints how sure the model was, from the recorded
probabilities. It never fails the test:

- for each no-skill prompt, `none`'s probability against the best skill's, so a
  narrow call is visible even when the model answered correctly;
- a sweep of `HOLSTERED_PICK_THRESHOLD` values with the `correct` and
  `abstained` counts each would score, to calibrate that setting.

## What `cargo test` checks

`eval.rs` (with its `harness/`) runs the real binary on every case, with `evals/skills` as the
skill library and a local mock as the decisions endpoint. The mock answers with
the recorded choice. For each model in `baseline.json` the test fails when:

- **a shortlist changed** since recording (retrieval, tokenising, the pool size,
  or the fixture skills changed), with the prompts listed and a note to
  re-record;
- **any score dropped** below the baseline.

## Re-recording

Re-record after changing retrieval, the request, a fixture, a case, or when
trying another model:

```bash
# Jev on OpenRouter
HOLSTERED_EVAL_RECORD=jev OPENROUTER_API_KEY=sk-or-... \
  cargo test --test eval -- --ignored record --nocapture

# a local Kev server
HOLSTERED_EVAL_RECORD=kev HOLSTERED_EVAL_UPSTREAM=http://localhost:8009/v1/systemone \
  cargo test --test eval -- --ignored record --nocapture
```

Recording rewrites that model's cassette and baseline entry, and prints every
miss. Commit both; the diff shows exactly which prompts changed and how the
scores moved, so a regression cannot slip in as a quiet re-record.

To add a case, append it to `cases.json` (and a skill under `skills/` if
needed), then re-record every model.
