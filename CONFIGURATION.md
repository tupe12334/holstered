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
| `HOLSTERED_THRESHOLD` | — | Lets runner-up skills in. The decision model scores every shortlisted skill and `none`, and its top pick is whichever scores highest. If that is `none`, nothing is injected; otherwise the pick is always injected, whatever its score, since the threshold never applies to it. Unset, the pick is injected alone. Set (e.g. `0.2`), every other shortlisted skill whose probability reaches it is injected after the pick, best first, up to 3 skills in total. |

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
