---
name: coc-tool
description: Use the Council backend helper to create missions, create tasks, and close missions through the authenticated internal API.
---

# coc-tool

Use this skill when you need to change Council state from OpenClaw.

## Safety Rules

- Use `director` credentials unless the task explicitly requires another agent id.
- Prefer `health` first when you are unsure whether the backend bridge is alive.
- Do not invent mission or task ids. Read them from command output before using them in follow-up commands.
- **Report logs** using `log-report` when you perform significant actions or encounter issues.
- **Report usage** using `usage-report` after every model call.

## Command Pattern

Run the helper from the read-only repo mirror with `sh -lc` through the `exec` tool.

```bash
agent_id="director"
agent_token="$(printf '%s' "$AGENT_TOKENS" | tr ',' '\n' | awk -F= -v id="$agent_id" '$1==id {print $2}')"
COC_BACKEND_BASE_URL="http://backend:8080" \
COC_AGENT_ID="$agent_id" \
COC_AGENT_TOKEN="$agent_token" \
/repo/scripts/coc-tool/coc-tool health
```

## Common Operations

Health check:
```bash
/repo/scripts/coc-tool/coc-tool health
```

Create mission:
```bash
/repo/scripts/coc-tool/coc-tool mission-create --title "Mission title" --description "Mission description"
```

Create task:
```bash
/repo/scripts/coc-tool/coc-tool task-create --title "Task title" --description "Task description" --priority normal --target-agent senior-engineer --mission-id mission-123
```

Claim task (mark as in-progress by you):
```bash
/repo/scripts/coc-tool/coc-tool task-claim --task-id task-123
```

Complete task:
```bash
/repo/scripts/coc-tool/coc-tool task-complete --task-id task-123 --notes "Task finished successfully"
```

Fail task:
```bash
/repo/scripts/coc-tool/coc-tool task-fail --task-id task-123 --reason "Detailed error message"
```

Close mission:
```bash
/repo/scripts/coc-tool/coc-tool mission-close --mission-id mission-123 --notes "Final notes"
```

Report log to live dashboard:
```bash
/repo/scripts/coc-tool/coc-tool log-report --level info --message "Working on database migration"
```

Report model usage for performance analytics:
```bash
/repo/scripts/coc-tool/coc-tool usage-report --model "openai/gpt-5.4" --prompt-tokens 500 --completion-tokens 200 --latency-ms 1200
```
