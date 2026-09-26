"""Hermes plugin for holstered: runs the holstered binary before each LLM turn."""

from __future__ import annotations

import json
import logging
import os
import subprocess
from typing import Any

logger = logging.getLogger(__name__)

# holstered caps its own decision-model call at 8s by default; this only guards a hung process.
TIMEOUT_SECONDS = 30


def _pre_llm_call(session_id: str = "", user_message: Any = None, **_: Any) -> dict | None:
    if not isinstance(user_message, str) or not user_message.strip():
        return None
    # Same stdin payload Hermes sends a pre_llm_call shell hook, so polyhook detects Hermes.
    payload = {
        "hook_event_name": "pre_llm_call",
        "tool_name": None,
        "tool_input": None,
        "session_id": session_id or "",
        "cwd": os.getcwd(),
        "extra": {"user_message": user_message},
    }
    try:
        run = subprocess.run(
            ["holstered"], input=json.dumps(payload), capture_output=True, text=True,
            timeout=TIMEOUT_SECONDS, check=False,
        )
        context = json.loads(run.stdout or "{}").get("context")
    except FileNotFoundError:
        logger.warning("holstered plugin: `holstered` is not on PATH; see https://github.com/tupe12334/holstered#install")
        return None
    except (OSError, subprocess.TimeoutExpired, ValueError, AttributeError) as exc:
        logger.warning("holstered plugin: skipped this turn: %s", exc)
        return None
    return {"context": context} if isinstance(context, str) and context.strip() else None


def register(ctx: Any) -> None:
    ctx.register_hook("pre_llm_call", _pre_llm_call)
