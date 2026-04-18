---
name: backend-audit
description: Read Council backend state through the read-only REST endpoints for health, audit, mission, council, usage, and task visibility.
---

# backend-audit

Use this skill when you need current Council state without mutating anything.

## Safety Rules

- Only use `GET` requests.
- Prefer backend APIs over direct datastore access.
- Start with `/api/health` or `/api/health/services` if you suspect the stack is degraded.

## Preferred Endpoints

- `http://backend:8080/api/health`
- `http://backend:8080/api/health/services`
- `http://backend:8080/api/overview`
- `http://backend:8080/api/tasks`
- `http://backend:8080/api/council`
- `http://backend:8080/api/history/missions`
- `http://backend:8080/api/audit`
- `http://backend:8080/api/usage`

## Command Pattern

Use `curl -fsS` through the `exec` tool.

```bash
curl -fsS http://backend:8080/api/health
```

When output is large, save it to a temporary file under `/tmp` and inspect the relevant slice with `sed`.
