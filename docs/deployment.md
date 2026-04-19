# Council of Claws: Deployment & Portability

The platform is designed to support two distinct handoff modes:

- `repo-first`: clone the repo and start clean state on the next machine.
- `resume-state`: move a sanitized `data/` snapshot when you want exact continuity.

## Prerequisites

### Minimal
- Docker with Docker Compose v2 (`docker compose`)
- `curl`
- `python3`
- `git`

### Bootstrap/build tools
- `make`
- Node.js + `npm`
- Rust + `cargo`

### Optional
- `CLOUDFLARED_TOKEN` in `.env` when using the optional tunnel profile
- AI provider keys in `.env` for whichever models OpenClaw should use
- `TELEGRAM_BOT_TOKEN` and `TELEGRAM_ALLOWED_USER_IDS` in `.env` when using the Telegram plugin

## Deployment Steps

1. **Environment Setup:**
   Copy `.env.example` to `.env` and fill in your API keys (Google, Groq, Telegram).
   ```bash
   make setup-env
   ```
   `OPENCLAW_GATEWAY_TOKEN` controls Control UI access and defaults to a local-only development token.
   Rotate `POSTGRES_PASSWORD`, `OPENCLAW_GATEWAY_TOKEN`, and `AGENT_TOKENS` before the first deploy on a new machine.
   Set `CLOUDFLARED_TOKEN` too if this machine should expose the optional tunnel profile.

2. **Preflight:**
   Verify Docker access, `.env`, writable data directories, and local ports:
   ```bash
   make preflight
   ```

   For a new-machine deploy, fail hard on placeholder secrets and unpinned images:
   ```bash
   make preflight-deploy
   ```

3. **Choose Image Source:**
   For remote GHCR deploys, set these in `.env` to the tags published by CI:
   - `BACKEND_IMAGE`
   - `DASHBOARD_IMAGE`
   - `GATEWAY_IMAGE`

   Example:
   - `ghcr.io/<owner>/council-of-claws-backend:main`
   - `ghcr.io/<owner>/council-of-claws-dashboard:main`
   - `ghcr.io/<owner>/council-of-claws-gateway:main`

   Then pull them explicitly:
   ```bash
   make compose-pull
   ```

   The workflow publishes to GHCR using the repository `GITHUB_TOKEN`. If the package visibility is private, the target machine must authenticate to GHCR before pulling.

   Optional GitHub Actions notifications:
   - `TELEGRAM_BOT_TOKEN` secret
   - `TELEGRAM_CHAT_ID` secret

   When both are configured, the workflow posts a Telegram summary covering checks, image build decisions, and image publish results.

4. **Start the Stack:**
   Fresh machine or first bring-up:
   ```bash
   make launch
   ```

   Fresh machine with the optional tunnel profile:
   ```bash
   make launch-tunnel
   ```

   Direct build/start without the bootstrap wrapper:
   ```bash
   make compose-up-build
   ```

   After the first build, subsequent starts normally use:
   ```bash
   make compose-up
   ```

   Start the optional Cloudflare tunnel profile only when you actually need remote ingress:
   ```bash
   make compose-up-build-tunnel
   ```
   `make launch-tunnel` and `make compose-up-build-tunnel` both read `.env`; `launch-tunnel` adds setup, preflight, and post-start verification.

5. **Verify:**
   Check the dashboard at `http://127.0.0.1:3000`.
   Open the OpenClaw Control UI from `http://127.0.0.1:3000/gateway` or print the raw tokenized URL with `make gateway-url`.
   Then run:
   ```bash
   make post-start-verify
   ```
   This verifies dashboard health, backend health, the tokenized OpenClaw URL, and an authenticated `coc-tool health` call.

## Runtime State

The runtime state lives under `data/`:
- `data/postgres`: Database records (Missions, Tasks, Agents).
- `data/redis`: Real-time state and streams.
- `data/obsidian`: Narrative Markdown memory.
- `data/openclaw`: Orchestrator internal state.

The repo source is separate from that runtime state. OpenClaw loads repo-managed config from `apps/gateway/config/openclaw.json5` and repo-local skills from `.agents/skills`, while its writable state continues to live in `data/openclaw`.

Additional runtime notes:
- PostgreSQL 18 is mounted at `data/postgres -> /var/lib/postgresql`, which matches the image's versioned storage layout.
- The gateway seeds `data/openclaw/config/openclaw.json5` from the repo bootstrap file on first boot only.
- After first boot, runtime edits to `data/openclaw/config/openclaw.json5` persist across restarts.
- To re-seed the gateway config from the repo bootstrap file, remove `data/openclaw/config/openclaw.json5` and restart the gateway.
- The gateway listens on `0.0.0.0` inside the container, while Compose publishes it on `127.0.0.1:${GATEWAY_PORT}` on the host. This is expected and compatible with the optional Cloudflare tunnel.

## Public Routes

When the tunnel is enabled, the public hostname is controlled by the Cloudflare Tunnel configuration associated with `CLOUDFLARED_TOKEN`, not by files in this repo. The externally useful paths are:

- Dashboard UI: `/`
- Dashboard health: `/health`
- Gateway redirect via dashboard: `/gateway`
- Raw OpenClaw gateway UI: `/?token=<OPENCLAW_GATEWAY_TOKEN>`
- Backend health is local-only by default: `http://127.0.0.1:8080/api/health`

`/gateway` is the preferred external entry point because the dashboard app redirects it to the tokenized raw gateway URL for the same hostname.

## Repo-First Handoff

1. Clone the repository.
2. Copy your `.env` file.
3. Run `make preflight-deploy`.
4. Set `BACKEND_IMAGE`, `DASHBOARD_IMAGE`, and `GATEWAY_IMAGE` in `.env` to the GHCR tags you want to deploy.
5. Run `make compose-pull`.
6. Run `make compose-up`.
This path gives you the latest published images and a clean runtime state.

## Resume Existing State

When you want the next machine to pick up exactly where the current one left off:

1. Export a sanitized snapshot:
   ```bash
   make state-export
   ```
2. Move the generated archive from `tmp/` plus your `.env` file to the new machine.
3. On the new machine, clone the repo and restore the snapshot:
   ```bash
   make state-import ARCHIVE=tmp/council-state-YYYYMMDDTHHMMSSZ.tar.gz
   ```
   Or run:
   ```bash
   bash scripts/dev/import-state.sh tmp/council-state-YYYYMMDDTHHMMSSZ.tar.gz
   ```
4. Start the stack with `make compose-up`.
5. Run `make post-start-verify`.

The export intentionally excludes transient OpenClaw logs, sqlite `-wal` and `-shm` files, config backups, and update-check noise so the handoff stays clean.
