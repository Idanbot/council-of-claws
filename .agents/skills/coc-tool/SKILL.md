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
agent_id="director"
agent_token="$(printf '%s' "$AGENT_TOKENS" | tr ',' '\n' | awk -F= -v id="$agent_id" '$1==id {print $2}')"
COC_BACKEND_BASE_URL="http://backend:8080" \
COC_AGENT_ID="$agent_id" \
COC_AGENT_TOKEN="$agent_token" \
/repo/scripts/coc-tool/coc-tool health
```

Create mission:

```bash
agent_id="director"
agent_token="$(printf '%s' "$AGENT_TOKENS" | tr ',' '\n' | awk -F= -v id="$agent_id" '$1==id {print $2}')"
COC_BACKEND_BASE_URL="http://backend:8080" \
COC_AGENT_ID="$agent_id" \
COC_AGENT_TOKEN="$agent_token" \
/repo/scripts/coc-tool/coc-tool mission-create "Mission title" "Mission description"
```

Create task:

```bash
agent_id="director"
agent_token="$(printf '%s' "$AGENT_TOKENS" | tr ',' '\n' | awk -F= -v id="$agent_id" '$1==id {print $2}')"
COC_BACKEND_BASE_URL="http://backend:8080" \
COC_AGENT_ID="$agent_id" \
COC_AGENT_TOKEN="$agent_token" \
/repo/scripts/coc-tool/coc-tool task-create "Task title" "Task description" normal senior-engineer mission-123
```

Close mission:

```bash
agent_id="director"
agent_token="$(printf '%s' "$AGENT_TOKENS" | tr ',' '\n' | awk -F= -v id="$agent_id" '$1==id {print $2}')"
COC_BACKEND_BASE_URL="http://backend:8080" \
COC_AGENT_ID="$agent_id" \
COC_AGENT_TOKEN="$agent_token" \
/repo/scripts/coc-tool/coc-tool mission-close mission-123 "Final notes"
```
