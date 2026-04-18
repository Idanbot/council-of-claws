#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "Stopping local stack and removing compose volumes..."
docker compose --env-file .env -f infra/compose/docker-compose.yml down -v || true

echo "Pruning unused Docker resources (images, containers, networks, volumes)..."
docker system prune -a --volumes -f

echo "Pruning build cache..."
docker builder prune -a -f

echo "Docker prune complete"
