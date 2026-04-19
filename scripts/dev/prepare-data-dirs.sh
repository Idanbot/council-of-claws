#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ensure_container_writable_dir() {
  local dir="$1"

  mkdir -p "$dir"

  # These bind-mounted runtime directories must be writable from containers
  # whose UIDs do not match the host user (for example, the OpenClaw node user
  # in CI). Keep the scope narrow to the mutable runtime paths.
  chmod 0777 "$dir"
}

ensure_container_writable_dir data/openclaw
ensure_container_writable_dir data/openclaw/config
ensure_container_writable_dir data/obsidian
ensure_container_writable_dir data/postgres
ensure_container_writable_dir data/workspace
