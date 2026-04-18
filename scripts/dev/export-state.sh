#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

mkdir -p tmp

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
archive="${1:-tmp/council-state-${stamp}.tar.gz}"

tar \
  --exclude='data/openclaw/logs/*' \
  --exclude='data/openclaw/tasks/*.sqlite-shm' \
  --exclude='data/openclaw/tasks/*.sqlite-wal' \
  --exclude='data/openclaw/config/*.bak*' \
  --exclude='data/openclaw/openclaw.json.bak*' \
  --exclude='data/openclaw/update-check.json' \
  -czf "$archive" \
  data

echo "Exported sanitized state snapshot to $archive"
echo "This archive does not include .env; move that file separately."
