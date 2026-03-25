#!/usr/bin/env python3
"""PreToolUse hook: block git commit unless runtime verification was performed.

Reads the conversation transcript and checks that, after the most recent
file edit (Edit/Write on a .py file), both of these happened:
  1. The app was executed  (Bash command containing 'uv run python')
  2. An HTTP check was made (Bash command containing 'curl')

If either is missing the commit is blocked with a message listing the
missing steps.
"""

import json
import sys


def _is_git_commit(command: str) -> bool:
    return "git commit" in command or "git " in command and "commit" in command


def main() -> None:
    data = json.load(sys.stdin)

    if data.get("stop_hook_active"):
        return

    tool_input = data.get("tool_input", {})
    command = tool_input.get("command", "")
    if not _is_git_commit(command):
        return

    transcript_path = data.get("transcript_path", "")
    if not transcript_path:
        return

    try:
        with open(transcript_path, encoding="utf-8") as f:
            lines = f.readlines()
    except OSError:
        return

    # Walk backwards through transcript entries.
    # Find the last Edit/Write on a .py file, then check if uv run + curl
    # appeared after it.
    last_py_edit_idx = -1
    for i in range(len(lines) - 1, -1, -1):
        try:
            entry = json.loads(lines[i])
        except json.JSONDecodeError:
            continue
        content = entry.get("message", {}).get("content", [])
        if not isinstance(content, list):
            continue
        for block in content:
            if not isinstance(block, dict):
                continue
            if block.get("type") != "tool_use":
                continue
            name = block.get("name", "")
            if name not in ("Edit", "Write"):
                continue
            inp = block.get("input", {})
            path = inp.get("file_path", "")
            if path.endswith(".py"):
                last_py_edit_idx = i
                break
        if last_py_edit_idx >= 0:
            break

    # No .py edit found — nothing to enforce
    if last_py_edit_idx < 0:
        return

    ran_app = False
    ran_curl = False

    for line in lines[last_py_edit_idx + 1 :]:
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        content = entry.get("message", {}).get("content", [])
        if not isinstance(content, list):
            continue
        for block in content:
            if not isinstance(block, dict):
                continue
            if block.get("type") != "tool_use":
                continue
            if block.get("name") != "Bash":
                continue
            cmd = block.get("input", {}).get("command", "")
            if "uv run python" in cmd:
                ran_app = True
            if "curl" in cmd:
                ran_curl = True

    missing = []
    if not ran_app:
        missing.append("- アプリ実行 (uv run python)")
    if not ran_curl:
        missing.append("- HTTP応答確認 (curl)")

    if missing:
        reason = (
            "ランタイム検証なしで commit しようとしています。\n"
            "以下の検証を先に実行してください:\n"
            + "\n".join(missing)
        )
        json.dump({"decision": "block", "reason": reason}, sys.stdout)


if __name__ == "__main__":
    main()
