#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CONFIG_FILE="apps/gateway/config/openclaw.json5"
ENV_EXAMPLE_FILE=".env.example"
COMPOSE_FILE="infra/compose/docker-compose.yml"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "Missing gateway config: $CONFIG_FILE"
  exit 1
fi

if ! grep -Eq 'token: "\$\{OPENCLAW_GATEWAY_TOKEN\}"' "$CONFIG_FILE"; then
  echo "Expected OPENCLAW_GATEWAY_TOKEN wiring not found in $CONFIG_FILE"
  exit 1
fi

if [[ ! -f "$ENV_EXAMPLE_FILE" ]]; then
  echo "Missing env template: $ENV_EXAMPLE_FILE"
  exit 1
fi

timezone_line="$(grep -E '^TIMEZONE=' "$ENV_EXAMPLE_FILE" || true)"
if [[ -z "$timezone_line" ]]; then
  echo "Missing TIMEZONE entry in $ENV_EXAMPLE_FILE"
  exit 1
fi

timezone_value="${timezone_line#TIMEZONE=}"
if [[ -n "$timezone_value" && "$timezone_value" != "UTC" ]]; then
  echo "TIMEZONE in $ENV_EXAMPLE_FILE must be UTC or empty"
  exit 1
fi

if ! grep -Eq '^OPENCLAW_GATEWAY_TOKEN=' "$ENV_EXAMPLE_FILE"; then
  echo "Missing OPENCLAW_GATEWAY_TOKEN entry in $ENV_EXAMPLE_FILE"
  exit 1
fi

if ! grep -Eq 'OPENCLAW_CONFIG_PATH: /home/node/.openclaw/config/openclaw.json5' "$COMPOSE_FILE"; then
  echo "Missing OPENCLAW_CONFIG_PATH wiring in $COMPOSE_FILE"
  exit 1
fi

if ! grep -Eq 'OPENCLAW_GATEWAY_TOKEN: \$\{OPENCLAW_GATEWAY_TOKEN:-council-local-gateway-token\}' "$COMPOSE_FILE"; then
  echo "Missing OPENCLAW_GATEWAY_TOKEN wiring in $COMPOSE_FILE"
  exit 1
fi

if ! grep -Eq 'context: ../../apps/gateway' "$COMPOSE_FILE"; then
  echo "Missing gateway image build context in $COMPOSE_FILE"
  exit 1
fi

if ! grep -Eq 'OPENCLAW_BASE_IMAGE:' "$COMPOSE_FILE"; then
  echo "Missing gateway base image build arg in $COMPOSE_FILE"
  exit 1
fi

if ! grep -Eq '/opt/council/bootstrap-config/openclaw.json5' "$COMPOSE_FILE"; then
  echo "Missing baked bootstrap config copy path in $COMPOSE_FILE"
  exit 1
fi

if ! grep -Eq '/repo/\.agents:ro' "$COMPOSE_FILE"; then
  echo "Missing project skills mount in $COMPOSE_FILE"
  exit 1
fi

for agent_id in contractor director architect senior-engineer junior-engineer intern; do
  if ! grep -Eq "id: \"$agent_id\"" "$CONFIG_FILE"; then
    echo "Missing agent '$agent_id' in $CONFIG_FILE"
    exit 1
  fi
done

for skill_id in coc-tool repo-readonly backend-audit obsidian-memory; do
  if [[ ! -f ".agents/skills/$skill_id/SKILL.md" ]]; then
    echo "Missing project skill .agents/skills/$skill_id/SKILL.md"
    exit 1
  fi
done

echo "Gateway config smoke test passed"
