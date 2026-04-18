#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ENV_FILE="${ENV_FILE:-.env}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing ${ENV_FILE}. Run 'make setup-env' first."
  exit 1
fi

get_env_value() {
  local key="$1"
  local default_value="${2:-}"
  local line
  line="$(grep -E "^${key}=" "$ENV_FILE" | tail -n1 || true)"
  if [[ -z "$line" ]]; then
    printf '%s\n' "$default_value"
    return
  fi
  printf '%s\n' "${line#*=}"
}

wait_for_http() {
  local name="$1"
  local url="$2"
  local retries="${3:-45}"
  until curl -fsS "$url" >/dev/null 2>&1; do
    retries=$((retries - 1))
    if [[ "$retries" -le 0 ]]; then
      echo "${name} did not become ready: ${url}"
      exit 1
    fi
    sleep 2
  done
}

dashboard_port="$(get_env_value DASHBOARD_PORT 3000)"
backend_port="$(get_env_value BACKEND_PORT 8080)"
gateway_port="$(get_env_value GATEWAY_PORT 18789)"
gateway_token="$(get_env_value OPENCLAW_GATEWAY_TOKEN council-local-gateway-token)"
agent_tokens="$(get_env_value AGENT_TOKENS "")"

director_token="$(printf '%s' "$agent_tokens" | tr ',' '\n' | awk -F= '$1=="director" {print $2}' | tail -n1)"
if [[ -z "$director_token" ]]; then
  echo "AGENT_TOKENS must include a director token for post-start verification"
  exit 1
fi

wait_for_http "dashboard health" "http://127.0.0.1:${dashboard_port}/health" 45
wait_for_http "backend health" "http://127.0.0.1:${backend_port}/api/health" 45
wait_for_http "OpenClaw gateway" "http://127.0.0.1:${gateway_port}/?token=${gateway_token}" 60

backend_health="$(curl -fsS "http://127.0.0.1:${backend_port}/api/health")"
if ! grep -Eq '"status"\s*:\s*"(ok|healthy)"' <<<"$backend_health"; then
  echo "Unexpected backend health payload: $backend_health"
  exit 1
fi

dashboard_health="$(curl -fsS "http://127.0.0.1:${dashboard_port}/health")"
if ! grep -Eq '"status"\s*:\s*"(ok|healthy)"' <<<"$dashboard_health"; then
  echo "Unexpected dashboard health payload: $dashboard_health"
  exit 1
fi

coc_health="$(
  COC_BACKEND_BASE_URL="http://127.0.0.1:${backend_port}" \
  COC_AGENT_ID="director" \
  COC_AGENT_TOKEN="$director_token" \
  python3 scripts/coc-tool/coc-tool health
)"

if ! grep -Eq '"status"\s*:\s*"(ok|healthy)"' <<<"$coc_health"; then
  echo "Authenticated coc-tool health check failed: $coc_health"
  exit 1
fi

echo "Post-start verification passed"
