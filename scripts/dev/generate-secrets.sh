#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ENV_FILE="${ENV_FILE:-.env}"
APPLY_MODE="${1:-}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing ${ENV_FILE}. Run 'make setup-env' first."
  exit 1
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1"
    exit 1
  fi
}

require_cmd python3

mapfile -t generated < <(python3 -c 'import secrets; print("OPENCLAW_GATEWAY_TOKEN=" + secrets.token_urlsafe(32)); print("AGENT_TOKENS=director=" + secrets.token_urlsafe(32))')

gateway_token_line="${generated[0]}"
agent_tokens_line="${generated[1]}"

if [[ "$APPLY_MODE" == "--apply" ]]; then
  bash scripts/dev/set-env-value.sh OPENCLAW_GATEWAY_TOKEN "${gateway_token_line#OPENCLAW_GATEWAY_TOKEN=}" >/dev/null
  bash scripts/dev/set-env-value.sh AGENT_TOKENS "${agent_tokens_line#AGENT_TOKENS=}" >/dev/null
fi

printf '%s\n' "$gateway_token_line"
printf '%s\n' "$agent_tokens_line"

if [[ "$APPLY_MODE" == "--apply" ]]; then
  echo "Updated ${ENV_FILE}"
fi
