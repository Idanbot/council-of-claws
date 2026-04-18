#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

# Default local stack: redis + postgres + backend + dashboard + gateway
# Cloudflare tunnel stays opt-in via the tunnel profile.
bash scripts/dev/preflight.sh
docker compose --env-file .env -f infra/compose/docker-compose.yml up -d
bash scripts/dev/post-start-verify.sh
