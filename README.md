# Council of Claws

[![CI/CD](https://github.com/Idanbot/council-of-claws/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Idanbot/council-of-claws/actions/workflows/ci.yml)
[![GHCR Images](https://img.shields.io/badge/ghcr-council--of--claws-2496ED.svg?logo=docker&logoColor=white)](https://github.com/users/Idanbot/packages?repo_name=council-of-claws)
[![Rust 1.89](https://img.shields.io/badge/rust-1.89-D34516.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Node 24](https://img.shields.io/badge/node-24-5FA04E.svg?logo=node.js&logoColor=white)](https://nodejs.org/)
[![OpenClaw 2026.4.15](https://img.shields.io/badge/openclaw-2026.4.15-0F766E.svg)](https://github.com/openclaw/openclaw)
[![PostgreSQL 18](https://img.shields.io/badge/postgres-18-4169E1.svg?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Redis 8](https://img.shields.io/badge/redis-8-DC382D.svg?logo=redis&logoColor=white)](https://redis.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-22C55E.svg)](LICENSE)

Real-time orchestration and observability for a multi-agent system powered by OpenClaw and a Rust control backend.

The Council of Claws is an autonomous platform powered by **OpenClaw** and a high-performance **Rust backend**. It manages complex multi-agent missions, provides real-time glassmorphic observability, and maintains a durable "Narrative Memory" in Obsidian.

## 🚀 Key Features

- **Autonomous Orchestration:** Multi-agent coordination using a contractor-director-worker layout in OpenClaw.
- **Glassmorphic Observability:** Real-time Svelte dashboard with WebSocket live-streams and animated agent status.
- **Durable Audit Trail:** Every agent decision and operation is persisted to PostgreSQL and streamed to Redis.
- **Narrative Memory:** Automated generation of human-readable Markdown summaries for every mission and task.
- **Advanced Auth & RBAC:** Argon2-backed backend identity verification and fine-grained scope profiles for each agent.
- **Project Skills:** Repo-local OpenClaw skills are loaded from `.agents/skills` and scoped per agent.
- **Pinned Gateway Image:** The gateway service builds from a small project-owned wrapper image pinned to a validated OpenClaw base digest.
- **Telegram Optional:** Telegram support is auto-enabled by OpenClaw when `TELEGRAM_BOT_TOKEN` is configured.

## 🛠 Tech Stack

- **Orchestrator:** [OpenClaw](https://github.com/openclaw/openclaw) (Multi-model Gateway)
- **Backend:** Rust (Axum, SQLx, Tokio, Redis-rs)
- **Dashboard:** SvelteKit 5 + Tailwind CSS 4
- **Database:** PostgreSQL 18 + Redis 8
- **Documentation:** Obsidian (Narrative Vault)
- **Infrastructure:** Docker Compose + Cloudflare Tunnels

## ✅ Prerequisites

### Minimal requirements

These are the tools needed to clone the repo, build the local stack, and run the built-in verification flow:

- `docker` with Docker Compose v2 (`docker compose`)
- `curl`
- `python3`
- `git`

### Project bootstrap tools

These are used by `make setup`, `make launch`, and `make launch-tunnel` to prepare local app dependencies before the Docker build:

- `node` and `npm`
- `rust` and `cargo`
- `make`

### Optional requirements

These are only needed for specific modes or workflows:

- `CLOUDFLARED_TOKEN` in `.env` for the optional tunnel profile (`make launch-tunnel`, `make compose-up-tunnel`, `make compose-up-build-tunnel`)
- `AGENT_HEARTBEAT_TTL_SECS` in `.env` to control stale-agent eviction from runtime telemetry (`120` seconds default)
- AI provider keys in `.env` for whichever upstream models you actually want OpenClaw to use
- Optional GitHub Copilot auth in `.env` via `COPILOT_GITHUB_TOKEN` (preferred), `GH_TOKEN`, or `GITHUB_TOKEN`, or interactively via `make copilot-login`
- `TELEGRAM_BOT_TOKEN` and `TELEGRAM_ALLOWED_USER_IDS` in `.env` if you want the Telegram plugin enabled
- `TELEGRAM_BOT_TOKEN`, `TELEGRAM_ALLOWED_USER_IDS`, and `DEPLOY_NOTIFICATION_ENABLED=true` in `.env` if you want an optional Telegram deploy notification after a successful verified launch
  The local deploy notification uses only the first ID from `TELEGRAM_ALLOWED_USER_IDS` if multiple IDs are comma-separated.

## 🏁 Getting Started

1. **Environment:**
   ```bash
   make setup-env
   ```
   Fill in the provider keys you actually use in `.env`. `OPENCLAW_GATEWAY_TOKEN` protects the OpenClaw Control UI.
   Before moving to a new machine, rotate `POSTGRES_PASSWORD`, `OPENCLAW_GATEWAY_TOKEN`, and `AGENT_TOKENS`.

   If you want the optional Cloudflare tunnel profile, set `CLOUDFLARED_TOKEN` too.

   Run the preflight before startup:
   ```bash
   make preflight
   ```

   For a new-machine deploy with real secrets:
   ```bash
   make preflight-deploy
   ```

2. **Launch:**
   Fresh machine or first local bring-up:
   ```bash
   make launch
   ```
   This runs:
   - `make setup`
   - `make preflight`
   - `make compose-up-build`
   - `make post-start-verify`
   - `make post-start-notify` when deploy notifications are enabled
   - `make smoke-e2e-flows` to verify critical agent lifecycles

## 🛠 Admin & Registry

The Council now features a dynamic **Skill Registry** and a file-based **Admin Control** panel:

1. **Dynamic Skills**: New skills added to `.agents/skills` are automatically discovered by the backend. Agents can query available capabilities at runtime.
2. **Admin UI**: Accessible at `/admin`, providing raw access to `openclaw.json5` and a live view of the skill registry. 
3. **E2E Validation**: Run `make smoke-e2e-flows` to simulate full agent loops (Director -> Junior task handoff, Obsidian design persistence, and schema generation).

   Fresh machine with the optional tunnel profile:
   ```bash
   make launch-tunnel
   ```
   This runs the same bootstrap flow, but with the Compose `tunnel` profile, an extra preflight check that `CLOUDFLARED_TOKEN` is present, and the same optional post-start notification step.

   If you only want to build and start the stack directly:
   ```bash
   make compose-up-build
   ```

   If you want a full serial restart that avoids compose race conditions on rapid rebuilds:
   ```bash
   make relaunch
   ```

   After the first successful build, normal restarts can use:
   ```bash
   make compose-up
   ```

   If you also want the optional Cloudflare tunnel profile:
   ```bash
   make compose-up-build-tunnel
   ```
   Serial restart with tunnel profile:
   ```bash
   make relaunch-tunnel
   ```
   `make compose-up-tunnel` and `make compose-up-build-tunnel` use the same profile; both read `.env` through Docker Compose.

3. **Access:**
   - **Dashboard:** `http://localhost:3000`
   - **Gateway UI:** `http://localhost:3000/gateway`
   - **Raw Gateway URL:** `http://localhost:18789/#token=<OPENCLAW_GATEWAY_TOKEN>`
   - **Health API:** `http://localhost:8080/api/health`
   - Prefer `/gateway` from the dashboard. The dashboard route redirects to the raw gateway URL with the shared token bootstrap attached.

4. **Verify After Start:**
   ```bash
   make post-start-verify
   ```
   This logs each verification step, waits for the dashboard, backend, and tokenized OpenClaw URL to come up, validates the health payloads, and runs an authenticated `coc-tool health` call.

   Run dashboard component tests (Agents and System pages included):
   ```bash
   make dashboard-test
   ```

5. **Generate Fresh Secrets:**
   Print fresh values without changing `.env`:
   ```bash
   make generate-secrets
   ```

   Generate fresh values, write them into `.env`, and print the applied values:
   ```bash
   make generate-secrets-apply
   ```

   These targets generate:
   - `OPENCLAW_GATEWAY_TOKEN`
   - `AGENT_TOKENS=director=...`

6. **Approve First Remote Browser/Device Pairing:**
   Remote Control UI sessions are not auto-approved. The first time you open the Control UI from a non-localhost browser profile, OpenClaw creates a pending pairing request that must be approved once.

   List pending requests:
   ```bash
   make devices-list
   ```

   Approve the newest pending request:
   ```bash
   make devices-approve-latest
   ```

   Or approve a specific request ID:
   ```bash
   make devices-approve REQUEST_ID=<request-id>
   ```

   After approval, re-open the Control UI from the dashboard route or the raw gateway URL.

7. **Model Providers And Fallbacks:**
   The gateway now has an explicit repo-managed model pool and per-agent priority chain in:
   - `apps/gateway/config/openclaw.json5`
   - `data/openclaw/config/openclaw.json5`

   Wired providers:
   - `openai` from `OPENAI_API_KEY`
   - `google` from `GEMINI_API_KEY`
   - `groq` from `GROQ_API_KEY`
   - optional `github-copilot` from `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_TOKEN`, or interactive device login

   Current shared model pool includes:
   - `openai/gpt-5.4`
   - `openai/gpt-5.4-mini`
   - `google/gemini-3.1-pro-preview`
   - `google/gemini-3-flash-preview`
   - `groq/llama-3.3-70b-versatile`
   - `groq/llama-3.1-8b-instant`
   - optional `github-copilot/gpt-4o`
   - optional `github-copilot/gpt-4.1`

   Useful inspection commands:
   ```bash
   make models-status
   make models-status AGENT=director
   make models-list
   make models-list PROVIDER=groq
   ```

   To connect GitHub Copilot interactively and persist it into the gateway auth store:
   ```bash
   make copilot-login
   ```

   The Copilot login writes into the persisted OpenClaw data directory, so it survives container restarts.

8. **Optional Telegram Deploy Notification:**
   You can enable a one-time Telegram message after `make launch` or `make launch-tunnel` finishes.

   Turn it on:
   ```bash
   make deploy-notification-on
   ```

   Turn it off:
   ```bash
   make deploy-notification-off
   ```

   Required `.env` keys:
   - `TELEGRAM_BOT_TOKEN`
   - `TELEGRAM_ALLOWED_USER_IDS`
   - `DEPLOY_NOTIFICATION_ENABLED=true`

   If `TELEGRAM_ALLOWED_USER_IDS` contains multiple comma-separated IDs, the local deploy notification sends only to the first one.

   Manual trigger:
   ```bash
   make post-start-notify
   ```

   `make post-start-notify` waits briefly for Compose health to settle. If Docker health is still catching up but a recent `make post-start-verify` succeeded, the notification is still sent and the message marks the still-starting services clearly.

   The message includes a timestamp, compose service readiness summary, local URLs, public URLs when configured, and a reminder about first-time remote pairing approval.

## ⚙️ Runtime Notes

- The stack uses PostgreSQL 18. Its data directory is bind-mounted at `data/postgres -> /var/lib/postgresql`, which matches the Postgres 18 image layout.
- Dashboard container now includes a healthcheck (`/health`) so Compose health reflects dashboard readiness.
- The gateway seeds `data/openclaw/config/openclaw.json5` from the repo bootstrap file on first boot only.
- After first boot, runtime edits to `data/openclaw/config/openclaw.json5` persist across restarts.
- The tradeoff is that later repo edits to `apps/gateway/config/openclaw.json5` do not automatically overwrite an existing runtime file. To re-seed from the repo version, remove `data/openclaw/config/openclaw.json5` and restart the gateway.
- The gateway process listens on `0.0.0.0` inside the container, while Compose publishes it on `127.0.0.1:${GATEWAY_PORT}` on the host. This is expected and compatible with the optional `cloudflared` tunnel profile.
- Public browser origins and trusted proxy CIDR are driven from `.env` through:
  - `PUBLIC_GATEWAY_ORIGIN`
  - `PUBLIC_DASHBOARD_ORIGIN`
  - `PUBLIC_DASHBOARD_INSECURE_ORIGIN`
  - `TRUSTED_PROXY_CIDR`
- Runtime telemetry is split between:
  - `dash:agents:status` (live heartbeat cache only; stale entries expire using `AGENT_HEARTBEAT_TTL_SECS`)
  - `dash:agents:configured` (configured roster cache)

### Runtime APIs

- `GET /api/agents/status` returns configured-vs-live diff with stale markers and heartbeat age.
- `GET /api/models/status` returns provider configuration status and configured model assignments per agent.
- `GET /api/admin/runtime-status` returns backend runtime summary, gateway reachability, provider readiness, and backend log tail.

## 🧩 Autonomous Skills

Project-scoped OpenClaw skills live in `.agents/skills` and are loaded into the gateway from the repo mirror mounted at `/repo`:
- **`coc-tool`**: Mission and task creation through the authenticated backend bridge.
- **`repo-readonly`**: Read-only inspection of the mounted repository.
- **`backend-audit`**: Read-only health, mission, council, task, usage, and audit inspection through backend APIs.
- **`obsidian-memory`**: Read-only inspection of the mounted Obsidian vault.

The remote worker compose profile is still a placeholder and is not part of the v1 path.

## 🐳 Image Publishing

GitHub Actions now publishes app images to GitHub Container Registry after checks pass:
- `council-of-claws-backend`
- `council-of-claws-dashboard`
- `council-of-claws-gateway`

The images are published under:
- `ghcr.io/<owner>/council-of-claws-backend`
- `ghcr.io/<owner>/council-of-claws-dashboard`
- `ghcr.io/<owner>/council-of-claws-gateway`

The workflow uses the repository `GITHUB_TOKEN`, so you do not need separate Docker Hub credentials. For remote deploys, set `BACKEND_IMAGE`, `DASHBOARD_IMAGE`, and `GATEWAY_IMAGE` in `.env` to the GHCR tags you want to pull, then run:
```bash
make compose-pull
make compose-up
```

If the repository or packages are private, the remote machine will need GHCR credentials to pull them.

Optional workflow notifications:
- `TELEGRAM_BOT_TOKEN` GitHub secret
- `TELEGRAM_CHAT_ID` GitHub secret

When both are set, the workflow sends a Telegram summary with the check results, per-image build decisions, and per-image publish status.

## 📦 Handoff

- **Repo-first handoff:** push the repo, copy `.env`, and start fresh with `make launch` on the next machine.
- **Tunnel handoff:** if the next machine should expose the tunnel too, copy `.env` with `CLOUDFLARED_TOKEN` set and use `make launch-tunnel`.
- **Deploy guardrails:** run `make preflight-deploy` before first start on the new machine, and `make post-start-verify` right after startup.
- **Resume exact runtime state:** export a sanitized snapshot with `make state-export`, move the archive plus `.env`, then restore with `make state-import ARCHIVE=tmp/council-state-YYYYMMDDTHHMMSSZ.tar.gz`.

## 📖 Documentation

- [Architecture Overview](docs/architecture.md)
- [Database & Stream Schema](docs/schema.md)
- [Deployment & Portability](docs/deployment.md)

---
*Created by agents, for agents.*
