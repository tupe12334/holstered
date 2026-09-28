# Configuration

holstered is configured through environment variables, set in the environment
your agent starts from.

| Variable | Default | Purpose |
|---|---|---|
| `OPENROUTER_API_KEY` | — | Required for Jev. Read from the environment only; never logged. |
| `HOLSTERED_SKILLS_DIRS` | `~/.claude/skills`, `~/.codex/skills`, `~/.agents/skills`, `~/.gemini/skills`, `~/.hermes/skills` | PATH-style list of skill roots. Any `SKILL.md` with a `description` in its frontmatter counts, up to 4 levels deep. |
| `HOLSTERED_JEV_URL` | `https://openrouter.ai/api/alpha/decisions` | Decisions endpoint. |
| `HOLSTERED_KEV_URL` | — | Use a local [Kev](#local-model-kev) server instead of Jev, e.g. `http://localhost:8009/v1/systemone`. No key needed; takes precedence over Jev. |
| `HOLSTERED_TIMEOUT_MS` | `8000` | How long to wait for the decision before letting the prompt through untouched. |
| `HOLSTERED_PICK_THRESHOLD` | — | The probability the decision model's pick needs to be injected. Under it, nothing is injected and the prompt goes through untouched, as if the model had answered `none`. Unset, any pick is injected. No default: on the [eval](evals/README.md#confidence-report) no threshold up to 0.7 changes Jev's picks, and from 0.5 up Kev loses at least one correct pick for every abstention it gains. |
| `HOLSTERED_RUNNER_UP_THRESHOLD` | `0.2` on Jev, `0.15` on Kev | The decision model's pick (its highest-scored option, `none` included) is always injected, followed by up to 2 runner-up skills whose probability reaches this, best first. Kev spreads its scores flatter than Jev: on a two-task prompt Jev scored 0.35 / 0.25 / 0.17 and Kev 0.21 / 0.16 / 0.16, hence the lower Kev default. Set it above `1` to inject the pick only. |

The model is `~typesafe/jev-latest` on Jev and `kev-latest` on Kev. When the key is set, the prompt (first
2,000 characters) and the shortlisted skill descriptions are sent to
OpenRouter. Each prompt with a shortlist takes about 0.7–1s longer.

## Local model: Kev

[Kev](https://github.com/jaredpalmer/kev) is an open-weights decision model that
serves the same System One API, so holstered can run fully offline: prompts
and skill descriptions never leave the machine.

```bash
git clone https://github.com/jaredpalmer/kev && cd kev
uv run --extra serve python -m kev.serve --run jaredpalmer/kev-4b --port 8009
export HOLSTERED_KEV_URL=http://localhost:8009/v1/systemone
```

On an Apple Silicon Mac, a live spot check of Kev-4B over a ~580-skill library
picked a fitting skill for 3/3 task prompts and stayed silent on an off-topic
one, at about 2–3.5s per prompt instead of Jev's ~1s. Its first request after
start loads the model (~30s) and outruns the default 8s timeout, so that prompt
goes through untouched; set `HOLSTERED_TIMEOUT_MS=45000` to wait it out instead.
Keep it under your agent's own hook timeout (60s in Claude Code by default).
Kev's README puts it a few points below Jev.
