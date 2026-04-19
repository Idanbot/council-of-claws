#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ENV_FILE="${ENV_FILE:-.env}"
VERIFY_STAMP_FILE="${VERIFY_STAMP_FILE:-/tmp/council-of-claws-post-start-verify.ok}"

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

require_cmd curl
require_cmd python3

log() {
  printf '[post-start-verify] %s\n' "$1"
}

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
  local total_retries="$retries"
  local attempts=1

  log "Checking ${name}: ${url}"
  until curl -fsS "$url" >/dev/null 2>&1; do
    retries=$((retries - 1))
    if [[ "$retries" -le 0 ]]; then
      echo "${name} did not become ready after ${attempts} attempts: ${url}"
      exit 1
    fi
    if (( attempts == 1 || attempts % 5 == 0 )); then
      log "${name} not ready yet; waiting (attempt ${attempts}/${total_retries})"
    fi
    attempts=$((attempts + 1))
    sleep 2
  done
  log "${name} ready after ${attempts} attempt(s)"
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

log "Starting post-start verification"
wait_for_http "dashboard health" "http://127.0.0.1:${dashboard_port}/health" 45
wait_for_http "backend health" "http://127.0.0.1:${backend_port}/api/health" 45
wait_for_http "OpenClaw gateway" "http://127.0.0.1:${gateway_port}/?token=${gateway_token}" 60

log "Validating backend health payload"
backend_health="$(curl -fsS "http://127.0.0.1:${backend_port}/api/health")"
if ! grep -Eq '"status"\s*:\s*"(ok|healthy)"' <<<"$backend_health"; then
  echo "Unexpected backend health payload: $backend_health"
  exit 1
fi
log "Backend health payload looks good"

log "Validating dashboard health payload"
dashboard_health="$(curl -fsS "http://127.0.0.1:${dashboard_port}/health")"
if ! grep -Eq '"status"\s*:\s*"(ok|healthy)"' <<<"$dashboard_health"; then
  echo "Unexpected dashboard health payload: $dashboard_health"
  exit 1
fi
log "Dashboard health payload looks good"

log "Running authenticated coc-tool health check"
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

printf '%s\n' "$(date -u +%s)" > "$VERIFY_STAMP_FILE"
log "Authenticated coc-tool health looks good"
log "Wrote verification stamp: ${VERIFY_STAMP_FILE}"
echo "Post-start verification passed"
