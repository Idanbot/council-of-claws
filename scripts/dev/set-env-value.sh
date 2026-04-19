#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ENV_FILE="${ENV_FILE:-.env}"
KEY="${1:-}"
VALUE="${2:-}"

if [[ -z "$KEY" ]]; then
  echo "Usage: bash scripts/dev/set-env-value.sh KEY VALUE"
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing ${ENV_FILE}. Run 'make setup-env' first."
  exit 1
fi

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

awk -v key="$KEY" -v value="$VALUE" '
BEGIN { updated = 0 }
$0 ~ ("^" key "=") {
  print key "=" value
  updated = 1
  next
}
{ print }
END {
  if (updated == 0) {
    print key "=" value
  }
}
' "$ENV_FILE" > "$tmp_file"

mv "$tmp_file" "$ENV_FILE"
echo "Set ${KEY} in ${ENV_FILE}"
