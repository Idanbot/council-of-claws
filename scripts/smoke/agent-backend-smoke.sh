#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

COMPOSE_FILE="infra/compose/docker-compose.yml"
PROJECT_NAME="council-of-claws"

export CLOUDFLARED_TOKEN="${CLOUDFLARED_TOKEN:-smoke-token}"
export AGENT_TOKENS="${AGENT_TOKENS:-director=smoke-director-token}"

agent_token="${AGENT_TOKENS#director=}"

cleanup() {
  docker compose -f "$COMPOSE_FILE" down -v || true
}
trap cleanup EXIT

dump_backend_logs() {
  echo "--- backend logs ---"
  docker compose -f "$COMPOSE_FILE" logs --tail=200 backend || true
  echo "--- backend state ---"
  docker inspect "$backend_container" --format '{{.State.Status}} {{.State.ExitCode}} {{.State.Error}}' || true
}

docker compose -f "$COMPOSE_FILE" up -d --build redis postgres backend

backend_container="${PROJECT_NAME}-backend-1"
NETWORK_NAME="$(docker inspect "$backend_container" --format '{{range $k, $v := .NetworkSettings.Networks}}{{println $k}}{{end}}' | head -n1 | tr -d '[:space:]')"
if [[ -z "$NETWORK_NAME" ]]; then
  echo "Failed to discover compose network for $backend_container"
  exit 1
fi

BACKEND_IP="$(docker inspect "$backend_container" --format '{{with index .NetworkSettings.Networks "'"$NETWORK_NAME"'"}}{{.IPAddress}}{{end}}')"
if [[ -z "$BACKEND_IP" ]]; then
  retries=30
  until [[ "$retries" -le 0 ]]; do
    BACKEND_IP="$(docker inspect "$backend_container" --format '{{with index .NetworkSettings.Networks "'"$NETWORK_NAME"'"}}{{.IPAddress}}{{end}}')"
    if [[ -n "$BACKEND_IP" ]]; then
      break
    fi
    sleep 1
    retries=$((retries - 1))
  done
  if [[ -z "$BACKEND_IP" ]]; then
    echo "Failed to discover backend container IP"
    exit 1
  fi
fi

BACKEND_BASE_URL="http://${BACKEND_IP}:8080"

for service in redis postgres; do
  retries=30
  until [[ "$retries" -le 0 ]]; do
    status="$(docker inspect --format='{{.State.Health.Status}}' "${PROJECT_NAME}-${service}-1" 2>/dev/null || true)"
    if [[ "$status" == "healthy" ]]; then
      break
    fi
    sleep 2
    retries=$((retries - 1))
  done

  if [[ "$retries" -le 0 ]]; then
    echo "Service $service did not become healthy in time"
    exit 1
  fi
done

backend_ready=false
for _ in $(seq 1 30); do
  if docker run --rm --network "$NETWORK_NAME" \
    -v "$ROOT_DIR:/workspace" -w /workspace \
    -e COC_BACKEND_BASE_URL="$BACKEND_BASE_URL" \
    -e COC_AGENT_ID=director \
    -e COC_AGENT_TOKEN="$agent_token" \
    python:3.13-alpine python scripts/coc-tool/coc-tool health >/dev/null; then
    backend_ready=true
    break
  fi
  sleep 2
done

if [[ "$backend_ready" != "true" ]]; then
  dump_backend_logs
  echo "Backend did not become ready in time"
  exit 1
fi

mission_response="$(docker run --rm --network "$NETWORK_NAME" \
  -v "$ROOT_DIR:/workspace" -w /workspace \
  -e COC_BACKEND_BASE_URL="$BACKEND_BASE_URL" \
  -e COC_AGENT_ID=director \
  -e COC_AGENT_TOKEN="$agent_token" \
  python:3.13-alpine python scripts/coc-tool/coc-tool mission-create --title "Bridge Smoke Mission" --description "Bridge smoke mission")"

mission_id="$(python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("mission_id",""))' <<<"$mission_response")"
if [[ -z "$mission_id" ]]; then
  echo "Failed to create mission: $mission_response"
  exit 1
fi

task_response="$(docker run --rm --network "$NETWORK_NAME" \
  -v "$ROOT_DIR:/workspace" -w /workspace \
  -e COC_BACKEND_BASE_URL="$BACKEND_BASE_URL" \
  -e COC_AGENT_ID=director \
  -e COC_AGENT_TOKEN="$agent_token" \
  python:3.13-alpine python scripts/coc-tool/coc-tool task-create --title "Bridge Smoke Task" --description "subtask" --priority high --target-agent architect --mission-id "$mission_id")"

task_id="$(python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("task_id",""))' <<<"$task_response")"
if [[ -z "$task_id" ]]; then
  echo "Failed to create subtask: $task_response"
  exit 1
fi

close_incomplete_response="$(docker run --rm --network "$NETWORK_NAME" \
  -v "$ROOT_DIR:/workspace" -w /workspace \
  -e COC_BACKEND_BASE_URL="$BACKEND_BASE_URL" \
  -e COC_AGENT_ID=director \
  -e COC_AGENT_TOKEN="$agent_token" \
  python:3.13-alpine python scripts/coc-tool/coc-tool mission-close --mission-id "$mission_id" --notes "first close attempt")"

close_incomplete_code="$(python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("code",""))' <<<"$close_incomplete_response")"
if [[ "$close_incomplete_code" != "MISSION_INCOMPLETE" ]]; then
  echo "Expected MISSION_INCOMPLETE, got: $close_incomplete_response"
  exit 1
fi

docker compose -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "${POSTGRES_USER:-council}" -d "${POSTGRES_DB:-council}" -v ON_ERROR_STOP=1 -c \
  "UPDATE tasks SET status='completed', updated_at=NOW() WHERE mission_id='${mission_id}' AND id='${task_id}';"

close_success_response="$(docker run --rm --network "$NETWORK_NAME" \
  -v "$ROOT_DIR:/workspace" -w /workspace \
  -e COC_BACKEND_BASE_URL="$BACKEND_BASE_URL" \
  -e COC_AGENT_ID=director \
  -e COC_AGENT_TOKEN="$agent_token" \
  python:3.13-alpine python scripts/coc-tool/coc-tool mission-close --mission-id "$mission_id" --notes "final close")"

close_status="$(python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("status",""))' <<<"$close_success_response")"
if [[ "$close_status" != "closed" ]]; then
  echo "Expected mission close success, got: $close_success_response"
  exit 1
fi

obsidian_file="data/obsidian/Missions/${mission_id}.md"
if [[ ! -f "$obsidian_file" ]]; then
  echo "Expected mission summary file not found: $obsidian_file"
  exit 1
fi

if ! grep -Fq "$task_id" "$obsidian_file"; then
  echo "Mission summary does not include task id $task_id"
  exit 1
fi

if ! grep -Fq "[[Tasks/${task_id}]]" "$obsidian_file"; then
  echo "Mission summary does not include subtask hyperlink [[Tasks/${task_id}]]"
  exit 1
fi

echo "Agent backend smoke test passed"
