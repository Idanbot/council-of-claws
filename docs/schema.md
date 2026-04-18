# Council of Claws: Database Schema

## PostgreSQL Tables

### `agents`
Durable identity and role definitions for autonomous entities.
- `id` (PK): Unique agent identifier (e.g., `director`).
- `display_name`: Human-readable name.
- `role`: Functional role.
- `enabled`: Boolean status.
- `scope_profile`: JSONB defining granular permissions (RBAC).
- `secret_hash`: Argon2 hash for API authentication.

### `tasks`
Atomic units of work.
- `id` (PK): Task identifier.
- `title`, `description`, `priority`, `status`.
- `owner_agent`: The agent currently assigned.
- `mission_id`: Link to parent mission.

### `missions`
High-level objectives encompassing multiple tasks.
- `id` (PK): Mission identifier.
- `root_task_id`: The primary task that initiated the mission.
- `status`: Active or Closed.

### `audit_events`
Immutable trail of all system actions.
- `id` (PK): Event identifier.
- `request_id`: For cross-service traceability.
- `operation`: e.g., `task_create`.
- `allowed`: Boolean (RBAC result).
- `metadata`: JSONB contextual data.

### `council_runs` & `council_votes`
Logic for multi-agent consensus.
- `council_runs`: Records of formal agent deliberations.
- `council_votes`: Individual votes (`approve`, `reject`) by agents.

---

## Redis Streams & Channels

- `coc:events:audit`: Durable stream of all audit logs.
- `coc:events:heartbeat`: Real-time stream of agent health and status.
- `coc:ws:broadcast`: Internal channel for WebSocket hub distribution.
