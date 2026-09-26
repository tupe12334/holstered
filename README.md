# holstered

**Hands your coding agent the right skill for each prompt.**

Agents with hundreds of skills see only their names, so the one that fits the
task often goes unused. holstered is a prompt hook: on every user prompt it
shortlists skills with BM25, asks the [Jev](https://openrouter.ai) decision
model to pick one (or none), and injects that skill's `SKILL.md` into the
model's context for the turn.

One Rust binary serves every agent through [polyhook](https://github.com/polyhook/polyhook),
which translates each agent's hook payload and response format.

## How it works

```
user prompt ─▶ agent prompt hook ─▶ holstered
                                      │ polyhook: detect agent, read prompt
                                      │ BM25 over name + description → top 20
                                      │ Jev (OpenRouter Decisions API): pick one or "none"
                                      │ polyhook: inject SKILL.md in the agent's format
                                      ▼
                            model sees the skill this turn
```

Nothing is injected, and the prompt goes through untouched, when Jev answers
`none`, names a skill it was not offered, the API fails or times out (8s by default), or
`OPENROUTER_API_KEY` is unset. holstered never blocks a prompt.

### Why BM25 + Jev

On a 38-prompt labeled set over a 581-skill library (32 prompts with a correct
skill, 6 with none):

| Pipeline | Correct pick | Stays silent on no-skill prompts |
|---|---|---|
| Keyword match | 14/32 | 5/6 |
| BM25 top-1 | 17/32 | 2/6 |
| BM25 top-20 → Cohere rerank-v3.5 | 25/32 | 2/6 |
| BM25 top-20 → Jev (Python prototype) | 28/32 | 6/6 |
| **holstered binary (`bm25` crate → Jev)** | **29/32** | **6/6** |

The holstered row is the release binary run end to end against live Jev
(median 915 ms per prompt). One of its three misses picked a `pptx` skill for a
slide-deck prompt the labels credited only to another skill. Small set, single labeler: read it as a direction, not a
guarantee.

## Supported agents

Agent support comes from polyhook; see its [Supported Tools](https://github.com/polyhook/polyhook#supported-tools)
for the full list and which agents can take context injection.

| Agent | Hook | Injection |
|---|---|---|
| Claude Code | `UserPromptSubmit` | ✅ |
| Codex | `UserPromptSubmit` | ✅ |
| Gemini CLI | `BeforeAgent` | ✅ |
| Hermes Agent | `pre_llm_call` | ✅ |
| Cline | `UserPromptSubmit` | ✅ |
| Cursor, Windsurf, Amp | — | ❌ their prompt hooks can only allow or block, so there is nothing to register |

## Install

```bash
cargo install holstered
export OPENROUTER_API_KEY=sk-or-...   # in the environment your agent starts from
```

Register it as the prompt hook of each agent you use.

**Claude Code** — `~/.claude/settings.json`
```json
{ "hooks": { "UserPromptSubmit": [ { "hooks": [ { "type": "command", "command": "holstered" } ] } ] } }
```

**Codex** — `~/.codex/hooks.json`
```json
{ "hooks": { "UserPromptSubmit": [ { "hooks": [ { "type": "command", "command": "holstered" } ] } ] } }
```

**Gemini CLI** — `~/.gemini/settings.json`
```json
{ "hooks": { "BeforeAgent": [ { "hooks": [ { "type": "command", "command": "holstered" } ] } ] } }
```

**Hermes Agent** — `~/.hermes/config.yaml` (approve it on first run, or set `hooks_auto_accept`)
```yaml
hooks:
  pre_llm_call:
    - command: holstered
```

**Cline** — hooks are executables named after the event
```bash
mkdir -p ~/Documents/Cline/Hooks
ln -s "$(command -v holstered)" ~/Documents/Cline/Hooks/UserPromptSubmit
```

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `OPENROUTER_API_KEY` | — | Required for Jev. Read from the environment only; never logged. |
| `HOLSTERED_SKILLS_DIRS` | `~/.claude/skills`, `~/.codex/skills`, `~/.agents/skills`, `~/.gemini/skills`, `~/.hermes/skills` | PATH-style list of skill roots. Any `SKILL.md` with a `description` in its frontmatter counts, up to 4 levels deep. |
| `HOLSTERED_JEV_URL` | `https://openrouter.ai/api/alpha/decisions` | Decisions endpoint. |
| `HOLSTERED_KEV_URL` | — | Use a local [Kev](#local-model-kev) server instead of Jev, e.g. `http://localhost:8009/v1/systemone`. No key needed; takes precedence over Jev. |
| `HOLSTERED_TIMEOUT_MS` | `8000` | How long to wait for the decision before letting the prompt through untouched. |

The model is `~typesafe/jev-latest` on Jev and `kev-latest` on Kev. When the key is set, the prompt (first
2,000 characters) and the shortlisted skill descriptions are sent to
OpenRouter. Each prompt with a shortlist takes about 0.7–1s longer.

### Local model: Kev

[Kev](https://github.com/jaredpalmer/kev) is an open-weights decision model that
serves the same System One API, so holstered can run fully offline: prompts
and skill descriptions never leave the machine.

```bash
git clone https://github.com/jaredpalmer/kev && cd kev
uv run --extra serve python -m kev.serve --run jaredpalmer/kev-4b --port 8009
export HOLSTERED_KEV_URL=http://localhost:8009/v1/systemone
```

On an Apple Silicon Mac, Kev-4B picked the same skills as Jev on a live spot
check, at about 2–3.5s per prompt instead of ~1s. Its first request after
start loads the model (~30s) and outruns the default 8s timeout, so that prompt
goes through untouched; set `HOLSTERED_TIMEOUT_MS=45000` to wait it out instead.
Keep it under your agent's own hook timeout (60s in Claude Code by default). Kev's README puts it a few points below Jev.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
