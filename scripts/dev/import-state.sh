#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

archive="${1:-}"
if [[ -z "$archive" ]]; then
  echo "Usage: $0 <snapshot.tar.gz>"
  exit 1
fi

if [[ ! -f "$archive" ]]; then
  echo "Snapshot not found: $archive"
  exit 1
fi

mkdir -p data
tar -xzf "$archive" -C "$ROOT_DIR"

echo "Imported state snapshot from $archive"
