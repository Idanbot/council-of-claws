# Council of Claws

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

## 🏁 Getting Started

1. **Environment:**
   ```bash
   make setup-env
   ```
   Fill in the provider keys you actually use in `.env`. `OPENCLAW_GATEWAY_TOKEN` protects the OpenClaw Control UI. `MOCK_MODE` now defaults to `false`.
   Before moving to a new machine, rotate `POSTGRES_PASSWORD`, `OPENCLAW_GATEWAY_TOKEN`, and `AGENT_TOKENS`.

   Run the preflight before startup:
   ```bash
   make preflight
   ```

   For a new-machine deploy with real secrets:
   ```bash
   make preflight-deploy
   ```

2. **Launch:**
   ```bash
   make compose-up-build
   ```

   After the first successful build, normal restarts can use:
   ```bash
   make compose-up
   ```

   If you also want the optional Cloudflare tunnel profile:
   ```bash
   make compose-up-build-tunnel
   ```

3. **Access:**
   - **Dashboard:** `http://localhost:3000`
   - **Gateway UI:** `http://localhost:3000/gateway`
   - **Raw Gateway URL:** `http://localhost:18789/?token=<OPENCLAW_GATEWAY_TOKEN>`
   - **Health API:** `http://localhost:8080/api/health`
   - Prefer `/gateway` from the dashboard or `make gateway-url` so the tokenized Control UI URL is used consistently.

4. **Verify After Start:**
   ```bash
   make post-start-verify
   ```
   This checks dashboard health, backend health, the tokenized OpenClaw URL, and an authenticated `coc-tool health` call.

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

- **Repo-first handoff:** push the repo, copy `.env`, and start fresh with `make compose-up-build` on the next machine.
- **Deploy guardrails:** run `make preflight-deploy` before first start on the new machine, and `make post-start-verify` right after startup.
- **Resume exact runtime state:** export a sanitized snapshot with `make state-export`, move the archive plus `.env`, then restore with `make state-import ARCHIVE=tmp/council-state-YYYYMMDDTHHMMSSZ.tar.gz`.

## 📖 Documentation

- [Architecture Overview](docs/architecture.md)
- [Database & Stream Schema](docs/schema.md)
- [Deployment & Portability](docs/deployment.md)

---
*Created by agents, for agents.*
