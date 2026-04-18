#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/dev/prepare-data-dirs.sh

ENV_FILE="${ENV_FILE:-.env}"
DEPLOY_MODE="false"
if [[ "${1:-}" == "--deploy" ]]; then
  DEPLOY_MODE="true"
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1"
    exit 1
  fi
}

get_env_value() {
  local key="$1"
  local default_value="${2:-}"
  local file="$3"
  local line
  line="$(grep -E "^${key}=" "$file" | tail -n1 || true)"
  if [[ -z "$line" ]]; then
    printf '%s\n' "$default_value"
    return
  fi
  printf '%s\n' "${line#*=}"
}

port_in_use() {
  local port="$1"
  if command -v ss >/dev/null 2>&1; then
    ss -ltn "( sport = :${port} )" | grep -q ":${port} "
    return
  fi
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
    return
  fi
  return 1
}

check_port_free() {
  local label="$1"
  local port="$2"
  if port_in_use "$port"; then
    echo "Port ${port} for ${label} is already in use"
    exit 1
  fi
}

check_writable_dir() {
  local dir="$1"
  mkdir -p "$dir"
  if [[ ! -w "$dir" ]]; then
    echo "Directory is not writable: $dir"
    exit 1
  fi
}

warn_if_inaccessible_dir() {
  local dir="$1"
  mkdir -p "$dir"
  if [[ ! -r "$dir" ]] || [[ ! -x "$dir" ]]; then
    echo "Warning: directory is not host-accessible: $dir"
  fi
}

ensure_not_placeholder() {
  local label="$1"
  local value="$2"
  local placeholder="$3"
  if [[ "$value" == "$placeholder" ]]; then
    echo "${label} is still using placeholder value '${placeholder}'"
    exit 1
  fi
}

require_cmd docker
docker info >/dev/null
docker compose version >/dev/null

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing ${ENV_FILE}. Run 'make setup-env' first."
  exit 1
fi

dashboard_port="$(get_env_value DASHBOARD_PORT 3000 "$ENV_FILE")"
backend_port="$(get_env_value BACKEND_PORT 8080 "$ENV_FILE")"
gateway_port="$(get_env_value GATEWAY_PORT 18789 "$ENV_FILE")"

check_port_free "dashboard" "$dashboard_port"
check_port_free "backend" "$backend_port"
check_port_free "gateway" "$gateway_port"

check_writable_dir data
check_writable_dir data/openclaw
check_writable_dir data/obsidian
check_writable_dir data/workspace
warn_if_inaccessible_dir data/postgres
warn_if_inaccessible_dir data/redis

if [[ "$DEPLOY_MODE" == "true" ]]; then
  postgres_password="$(get_env_value POSTGRES_PASSWORD change-me "$ENV_FILE")"
  gateway_token="$(get_env_value OPENCLAW_GATEWAY_TOKEN council-local-gateway-token "$ENV_FILE")"
  agent_tokens="$(get_env_value AGENT_TOKENS "" "$ENV_FILE")"
  openclaw_image="$(get_env_value OPENCLAW_IMAGE "" "$ENV_FILE")"

  ensure_not_placeholder "POSTGRES_PASSWORD" "$postgres_password" "change-me"
  ensure_not_placeholder "OPENCLAW_GATEWAY_TOKEN" "$gateway_token" "council-local-gateway-token"

  if [[ -z "$agent_tokens" ]] || grep -Eq 'change-me|example|director=$' <<<"$agent_tokens"; then
    echo "AGENT_TOKENS must be set to real rotated values before deploy"
    exit 1
  fi

  if [[ "$openclaw_image" == *":latest" ]]; then
    echo "OPENCLAW_IMAGE must be pinned, not :latest"
    exit 1
  fi
fi

echo "Preflight passed"
