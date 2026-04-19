#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ENV_FILE="${ENV_FILE:-.env}"
COMPOSE_FILE="infra/compose/docker-compose.yml"
VERIFY_STAMP_FILE="${VERIFY_STAMP_FILE:-/tmp/council-of-claws-post-start-verify.ok}"
NOTIFY_READY_WAIT_SECONDS="${NOTIFY_READY_WAIT_SECONDS:-90}"
NOTIFY_READY_POLL_SECONDS="${NOTIFY_READY_POLL_SECONDS:-3}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing ${ENV_FILE}. Run 'make setup-env' first."
  exit 1
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1"
    exit 1
  fi
}

require_cmd docker
require_cmd curl

log() {
  printf '[post-start-notify] %s\n' "$1"
}

get_env_value() {
  local key="$1"
  local default_value="${2:-}"
  local line
  line="$(grep -E "^${key}=" "$ENV_FILE" | tail -n1 || true)"
  if [[ -z "$line" ]]; then
    printf '%s\n' "$default_value"
    return
  fi
  printf '%s\n' "${line#*=}"
}

escape_html() {
  printf '%s' "$1" | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g'
}

collect_container_status() {
  local -n status_lines_ref=$1
  local -n all_ready_ref=$2
  local -n cloudflared_running_ref=$3
  local container_id service_name state_status health_status

  status_lines_ref=()
  all_ready_ref="true"
  cloudflared_running_ref="false"

  for container_id in "${container_ids[@]}"; do
    service_name="$(docker inspect "$container_id" --format '{{ index .Config.Labels "com.docker.compose.service" }}')"
    state_status="$(docker inspect "$container_id" --format '{{ .State.Status }}')"
    health_status="$(docker inspect "$container_id" --format '{{ if .State.Health }}{{ .State.Health.Status }}{{ else }}none{{ end }}')"

    if [[ "$service_name" == "cloudflared" && "$state_status" == "running" ]]; then
      cloudflared_running_ref="true"
    fi

    if [[ "$health_status" == "none" ]]; then
      if [[ "$state_status" == "running" ]]; then
        status_lines_ref+=("✅ <b>${service_name}</b>: running")
      else
        status_lines_ref+=("❌ <b>${service_name}</b>: ${state_status}")
        all_ready_ref="false"
      fi
    else
      if [[ "$health_status" == "healthy" ]]; then
        status_lines_ref+=("✅ <b>${service_name}</b>: healthy")
      elif [[ "$health_status" == "starting" ]]; then
        status_lines_ref+=("⚠️ <b>${service_name}</b>: starting")
        all_ready_ref="false"
      else
        status_lines_ref+=("❌ <b>${service_name}</b>: ${health_status}")
        all_ready_ref="false"
      fi
    fi
  done
}

recent_verify_success() {
  local max_age_seconds="${1:-900}"
  local now_ts verify_ts

  if [[ ! -f "$VERIFY_STAMP_FILE" ]]; then
    return 1
  fi

  verify_ts="$(tr -d '[:space:]' < "$VERIFY_STAMP_FILE")"
  if [[ -z "$verify_ts" ]] || ! [[ "$verify_ts" =~ ^[0-9]+$ ]]; then
    return 1
  fi

  now_ts="$(date -u +%s)"
  (( now_ts - verify_ts <= max_age_seconds ))
}

notification_enabled="$(get_env_value DEPLOY_NOTIFICATION_ENABLED false)"
if [[ "$notification_enabled" != "true" ]]; then
  log "Deploy notification disabled"
  exit 0
fi

telegram_bot_token="$(get_env_value TELEGRAM_BOT_TOKEN "")"
telegram_allowed_user_ids="$(get_env_value TELEGRAM_ALLOWED_USER_IDS "")"
if [[ -z "$telegram_bot_token" ]] || [[ -z "$telegram_allowed_user_ids" ]]; then
  log "Deploy notification skipped: TELEGRAM_BOT_TOKEN or TELEGRAM_ALLOWED_USER_IDS missing"
  exit 0
fi

mapfile -t container_ids < <(docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" ps -q)
if [[ "${#container_ids[@]}" -eq 0 ]]; then
  echo "No compose containers found"
  exit 1
fi

status_lines=()
all_ready="false"
cloudflared_running="false"
compose_ready_note=""
deadline_epoch="$(( $(date -u +%s) + NOTIFY_READY_WAIT_SECONDS ))"

log "Waiting up to ${NOTIFY_READY_WAIT_SECONDS}s for compose services to report ready"
while true; do
  collect_container_status status_lines all_ready cloudflared_running
  if [[ "$all_ready" == "true" ]]; then
    log "All compose services report ready"
    break
  fi

  if (( "$(date -u +%s)" >= deadline_epoch )); then
    break
  fi

  sleep "$NOTIFY_READY_POLL_SECONDS"
done

if [[ "$all_ready" != "true" ]]; then
  if recent_verify_success 900; then
    compose_ready_note="⚠️ <b>Compose health is still converging.</b> A recent <code>post-start-verify</code> run succeeded, so this notification was sent using verified endpoint readiness."
    log "Compose health is still converging; continuing because post-start verification passed recently"
  else
    log "Deploy notification aborted: not all compose containers are ready and no recent verification stamp was found"
    exit 1
  fi
fi

dashboard_port="$(get_env_value DASHBOARD_PORT 3000)"
gateway_port="$(get_env_value GATEWAY_PORT 18789)"
gateway_token="$(get_env_value OPENCLAW_GATEWAY_TOKEN council-local-gateway-token)"
public_dashboard_origin="$(get_env_value PUBLIC_DASHBOARD_ORIGIN "")"
public_dashboard_insecure_origin="$(get_env_value PUBLIC_DASHBOARD_INSECURE_ORIGIN "")"
public_gateway_url="$(get_env_value PUBLIC_GATEWAY_URL "")"

if [[ -z "$public_dashboard_origin" ]]; then
  public_dashboard_origin="http://127.0.0.1:${dashboard_port}"
fi

if [[ -z "$public_gateway_url" ]]; then
  public_gateway_url="http://127.0.0.1:${gateway_port}/#token=${gateway_token}"
fi

profile_name="local"
if [[ "$cloudflared_running" == "true" ]]; then
  profile_name="tunnel"
fi

run_date_utc="$(date -u +'%Y-%m-%d %H:%M:%S UTC')"
dashboard_url_local="http://127.0.0.1:${dashboard_port}"
gateway_url_local="http://127.0.0.1:${gateway_port}/#token=${gateway_token}"

status_block="$(printf '%s\n' "${status_lines[@]}")"
status_block="$(escape_html "$status_block")"
status_block="${status_block//&lt;b&gt;/<b>}"
status_block="${status_block//&lt;\/b&gt;/</b>}"

public_lines=()
public_lines+=("🌐 <b>Dashboard:</b> <code>$(escape_html "$public_dashboard_origin")</code>")
if [[ -n "$public_dashboard_insecure_origin" && "$public_dashboard_insecure_origin" != "$public_dashboard_origin" ]]; then
  public_lines+=("🌐 <b>Dashboard (HTTP):</b> <code>$(escape_html "$public_dashboard_insecure_origin")</code>")
fi
public_lines+=("🌐 <b>Gateway:</b> <code>$(escape_html "$public_gateway_url")</code>")

public_block="$(printf '%s\n' "${public_lines[@]}")"

msg="$(cat <<EOF
<b>📦 Council of Claws Stack Deployed</b>
━━━━━━━━━━━━━━━━━━━━
🕒 <b>Time:</b> <code>${run_date_utc}</code>
🧭 <b>Profile:</b> <code>${profile_name}</code>
📁 <b>Repo:</b> <code>$(escape_html "$ROOT_DIR")</code>
━━━━━━━━━━━━━━━━━━━━
✅ <b>Compose service status</b>
${status_block}
${compose_ready_note}
━━━━━━━━━━━━━━━━━━━━
🖥️ <b>Local Dashboard:</b> <code>$(escape_html "$dashboard_url_local")</code>
🛡️ <b>Local Gateway:</b> <code>$(escape_html "$gateway_url_local")</code>
${public_block}
━━━━━━━━━━━━━━━━━━━━
🔐 <b>Remote first-use note:</b> approve the pending OpenClaw pairing request if the Control UI is opened from a new remote browser/device.
EOF
)"

echo "::add-mask::$telegram_bot_token"

IFS=',' read -r -a telegram_targets <<<"$telegram_allowed_user_ids"
telegram_target="$(printf '%s' "${telegram_targets[0]:-}" | xargs)"
if [[ -z "$telegram_target" ]]; then
  log "Deploy notification skipped: first TELEGRAM_ALLOWED_USER_IDS entry is empty"
  exit 0
fi

curl -fsS -X POST "https://api.telegram.org/bot${telegram_bot_token}/sendMessage" \
  -d "chat_id=${telegram_target}" \
  -d "parse_mode=HTML" \
  -d "disable_web_page_preview=true" \
  --data-urlencode "text=${msg}" >/dev/null

log "Deploy notification sent"
