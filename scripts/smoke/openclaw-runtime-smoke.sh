#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/dev/prepare-data-dirs.sh

COMPOSE_FILE="infra/compose/docker-compose.yml"
TOKEN="${OPENCLAW_GATEWAY_TOKEN:-council-local-gateway-token}"
RUNTIME_CONFIG_PATH="/home/node/.openclaw/config/openclaw.json5"

docker compose --env-file .env -f "$COMPOSE_FILE" up -d redis postgres gateway

cleanup() {
  docker compose --env-file .env -f "$COMPOSE_FILE" down >/dev/null 2>&1 || true
}
trap cleanup EXIT

retries=90
until curl -fsS "http://127.0.0.1:18789/?token=$TOKEN" >/dev/null 2>&1; do
  retries=$((retries - 1))
  if [[ "$retries" -le 0 ]]; then
    echo "OpenClaw gateway did not become ready in time"
    docker compose --env-file .env -f "$COMPOSE_FILE" logs --tail=200 gateway || true
    exit 1
  fi
  sleep 2
done

if ! docker compose --env-file .env -f "$COMPOSE_FILE" exec -T gateway sh -lc "test -f '$RUNTIME_CONFIG_PATH'"; then
  echo "Expected runtime config not found at $RUNTIME_CONFIG_PATH inside gateway container"
  exit 1
fi

for agent_id in contractor director architect senior-engineer junior-engineer intern; do
  if ! docker compose --env-file .env -f "$COMPOSE_FILE" exec -T gateway sh -lc "grep -Eq '\"id\": \"$agent_id\"|id: \"$agent_id\"' '$RUNTIME_CONFIG_PATH'"; then
    echo "Runtime config missing agent '$agent_id'"
    exit 1
  fi
done

gateway_logs="$(docker compose --env-file .env -f "$COMPOSE_FILE" logs --tail=200 gateway)"

if grep -Eq 'Config invalid|EROFS|/workspace/gateway-config' <<<"$gateway_logs"; then
  echo "Unexpected gateway config failure detected"
  echo "$gateway_logs"
  exit 1
fi

if ! grep -Eq '\[gateway\] ready' <<<"$gateway_logs"; then
  echo "Gateway never reported ready"
  echo "$gateway_logs"
  exit 1
fi

echo "OpenClaw runtime smoke test passed"
