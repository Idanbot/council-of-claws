# OpenClaw Skills Reference (Legacy)

This file preserves the earlier shell-skill sketch that predated the current OpenClaw project-skill model.

It is a reference only. The active runtime now loads repo-local skills from `.agents/skills` via `skills.load.extraDirs`, and the current agent list lives in `apps/gateway/config/openclaw.json5`.

## Runtime Assumptions

- `COC_BACKEND_BASE_URL=http://backend:8080`
- PostgreSQL is reachable at `postgres`
- Redis is reachable at `redis`
- Obsidian is mounted at `/obsidian`
- Workspace is mounted at `/workspace`

## Skills

### `coc-tool`

Primary interface for the Council of Claws backend.
Supports: health, mission-create, task-create, mission-close.

```yaml
id: coc-tool
type: shell
path: /workspace/scripts/coc-tool/coc-tool
env:
  COC_BACKEND_BASE_URL: http://backend:8080
```

### `web-search`

Search the web and return AI-clean markdown results. No key required.

```yaml
id: web-search
type: shell
path: /usr/bin/curl
args: ["-s", "https://s.jina.ai/{{query}}"]
```

### `web-reader`

Read any URL and return clean markdown content stripped of ads/SEO. No key required.

```yaml
id: web-reader
type: shell
path: /usr/bin/curl
args: ["-s", "https://r.jina.ai/{{url}}"]
```

### `sql-explorer-ro`

Read-only access to the platform database for auditing.

```yaml
id: sql-explorer-ro
type: shell
path: /usr/bin/psql
env:
  PGPASSWORD: ${POSTGRES_PASSWORD}
args: ["-h", "postgres", "-U", "${POSTGRES_USER}", "-d", "${POSTGRES_DB}", "-c", "{{query}}"]
```

### `redis-explorer-ro`

Read-only inspection of the real-time event bus.

```yaml
id: redis-explorer-ro
type: shell
path: /usr/bin/redis-cli
args: ["-h", "redis", "get", "{{key}}"]
```

### `file-manager`

Repository file structure inspection.

```yaml
id: file-manager
type: shell
path: /usr/bin/ls
args: ["-R", "/workspace"]
```

### `obsidian-advanced`

Deep vault search and discovery.

```yaml
id: obsidian-advanced
type: shell
path: /usr/bin/find
args: ["/obsidian", "-name", "*{{pattern}}*"]
```

## Agent Assignments

### `contractor`

```yaml
role: front-door
model: cheap
can_create_tasks: true
can_run_shell: false
skills: [coc-tool, web-search, web-reader, file-manager]
```

### `director`

```yaml
role: planner
model: premium
can_create_tasks: true
can_run_shell: false
skills: [coc-tool, web-search, web-reader, sql-explorer-ro, redis-explorer-ro, obsidian-advanced]
```

### `architect`

```yaml
role: design-review
model: heavy
can_create_tasks: true
can_run_shell: false
skills: [coc-tool, obsidian-advanced, file-manager, web-reader]
```

### `senior_engineer`

```yaml
role: implementation
model: premium
fallback_model: heavy
can_create_tasks: true
can_run_shell: true
skills: [coc-tool, file-manager, web-search, web-reader]
```

### `junior_engineer`

```yaml
role: qa-review
model: cheap
fallback_model: premium
can_create_tasks: true
can_run_shell: true
skills: [coc-tool, file-manager, web-reader]
```

### `intern`

```yaml
role: notifications
model: cheap
can_create_tasks: true
can_run_shell: false
skills: [coc-tool, web-reader]
```

## Current Replacement

The supported v1 replacement is:

1. Repo-local skills under `.agents/skills`.
2. Current OpenClaw agent definitions in `apps/gateway/config/openclaw.json5`.
3. Backend and vault access through authenticated commands or read-only HTTP inspection rather than the older inline shell-skill block.
