# Council of Claws: System Architecture

The Council of Claws is a multi-agent orchestration platform designed for autonomous task management, real-time observability, and narrative memory.

## Core Components

### 1. OpenClaw Gateway (The Brain)
- **Role:** Orchestrates the multi-agent system.
- **Function:** Routes incoming requests from the Control UI and optional Telegram channel to named agents, manages agent state, and loads repo-local skills from `.agents/skills`.
- **Communication:** Connects to Redis for the real-time event bus and PostgreSQL for long-term memory.
- **Layout:** Uses a persistent mutable workspace at `/workspace` and a separate read-only repo mirror at `/repo`.
- **Config Model:** Seeds `data/openclaw/config/openclaw.json5` from the repo bootstrap file on first boot, then keeps the runtime file persistent across restarts.

### 2. Rust Backend (The Controller)
- **Role:** The central authority for the platform's state and business logic.
- **Function:** Manages Missions, Tasks, and Audit Logs. Provides the `coc-tool` skill used by agents.
- **Features:**
  - **Audit Service:** Durable logging of all agent operations.
  - **WebSocket Hub:** Real-time broadcasting of events to the dashboard.
  - **Auth:** Strict Argon2-based identity verification for agents.

### 3. Svelte Dashboard (The Eyes)
- **Role:** Real-time observability and control plane.
- **Function:** Displays live agent status, task progress, and audit trails using glassmorphic UI patterns.
- **Integration:** Communicates with the Backend via REST and WebSockets.

### 4. Data Layer (The Memory)
- **PostgreSQL:** Durable relational storage for Tasks, Missions, Agents, and Audit Events.
- **Redis:** Real-time event bus and durable streams for heartbeats and signals.
- **Obsidian:** Narrative memory. The system generates human-readable Markdown summaries of every mission and task.

## Data Flow

1. **Input:** User sends a request through the OpenClaw Control UI or optional Telegram channel.
2. **Front Door:** OpenClaw routes the initial request to the `contractor` agent.
3. **Planning:** The `contractor` or `director` decomposes the work and delegates to `architect`, `senior-engineer`, `junior-engineer`, or `intern` as needed.
4. **Action:** The `director` uses `coc-tool` to create a Mission and Tasks in the Backend.
5. **Broadcast:** The Backend persists the data to SQL, pushes an event to Redis, and broadcasts it over WebSockets.
6. **Observation:** The Dashboard updates instantly, showing the new Mission and Tasks.
7. **Execution:** Worker agents (`senior-engineer`, `junior-engineer`, etc.) claim tasks and report progress.
8. **Narrative:** Upon mission closure, the Backend generates a comprehensive summary in the Obsidian vault.

## Access Surfaces

- Dashboard UI: `/`
- Dashboard health: `/health`
- Gateway redirect route from the dashboard app: `/gateway`
- Raw tokenized OpenClaw UI: `/?token=<OPENCLAW_GATEWAY_TOKEN>`

The dashboard redirect route constructs the raw gateway URL for the current hostname, so it remains the safest stable UI entry point when the stack is accessed through the optional Cloudflare tunnel.
