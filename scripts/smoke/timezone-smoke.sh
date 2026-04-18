#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

COMPOSE_FILE="infra/compose/docker-compose.yml"
REMOTE_COMPOSE_FILE="infra/compose/docker-compose.remote-worker.yml"
ENV_EXAMPLE_FILE=".env.example"

if [[ ! -f "$ENV_EXAMPLE_FILE" ]]; then
  echo "Missing env template: $ENV_EXAMPLE_FILE"
  exit 1
fi

if ! grep -Eq '^TIMEZONE=' "$ENV_EXAMPLE_FILE"; then
  echo "Missing TIMEZONE in $ENV_EXAMPLE_FILE"
  exit 1
fi

required_patterns=(
  'redis:[[:space:][:print:]]*'
  'TZ: \$\{TIMEZONE:-UTC\}'
  'postgres:[[:space:][:print:]]*'
  'PGTZ: \$\{TIMEZONE:-UTC\}'
  'backend:[[:space:][:print:]]*'
  'TIMEZONE: \$\{TIMEZONE:-UTC\}'
  'dashboard:[[:space:][:print:]]*'
  'gateway:[[:space:][:print:]]*'
  'cloudflared:[[:space:][:print:]]*'
)

for pattern in "${required_patterns[@]}"; do
  if ! grep -Eq "$pattern" "$COMPOSE_FILE"; then
    echo "Missing timezone wiring pattern '$pattern' in $COMPOSE_FILE"
    exit 1
  fi
done

if ! grep -Eq 'TIMEZONE: \$\{TIMEZONE:-UTC\}' "$REMOTE_COMPOSE_FILE"; then
  echo "Missing TIMEZONE propagation in $REMOTE_COMPOSE_FILE"
  exit 1
fi

if ! grep -Eq 'TZ: \$\{TIMEZONE:-UTC\}' "$REMOTE_COMPOSE_FILE"; then
  echo "Missing TZ propagation in $REMOTE_COMPOSE_FILE"
  exit 1
fi

echo "Timezone smoke test passed"
