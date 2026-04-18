#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

COMPOSE_FILE="infra/compose/docker-compose.yml"

docker compose -f "$COMPOSE_FILE" config -q

docker compose -f "$COMPOSE_FILE" up -d redis postgres

cleanup() {
  docker compose -f "$COMPOSE_FILE" down -v || true
}
trap cleanup EXIT

for service in redis postgres; do
  retries=30
  until [[ "$retries" -le 0 ]]; do
    status="$(docker inspect --format='{{.State.Health.Status}}' "council-of-claws-${service}-1" 2>/dev/null || true)"
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

echo "Compose smoke test passed"
