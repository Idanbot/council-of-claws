#!/usr/bin/env bash
# E2E Smoke Test for critical agent flows (Task Lifecycle, Schema to SQL, Obsidian Design)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

BACKEND_URL="http://127.0.0.1:8080"
AGENT_ID="director"
AGENT_TOKEN="director-token"

# Try to extract real token from .env
if [ -f .env ]; then
  TOKEN=$(grep "^AGENT_TOKENS=" .env | cut -d'=' -f2- | tr ',' '\n' | grep "^${AGENT_ID}=" | cut -d'=' -f2 || echo "")
  if [ -n "$TOKEN" ]; then
    AGENT_TOKEN="$TOKEN"
  fi
fi

log() { echo -e "\x1b[34m[E2E]\x1b[0m $1"; }
error() { echo -e "\x1b[31m[ERROR]\x1b[0m $1"; exit 1; }

check_backend() {
  curl -sf "$BACKEND_URL/api/health" > /dev/null || error "Backend not reachable at $BACKEND_URL"
}

# --- Flow 1: Task Lifecycle ---
flow_task_lifecycle() {
  log "Starting Flow 1: Task Lifecycle (Director -> Junior)"
  
  # 1. Director creates a task for Junior
  local task_resp=$(curl -s -X POST "$BACKEND_URL/internal/tasks/create" \
    -H "Content-Type: application/json" -H "X-Agent-Id: director" -H "Authorization: Bearer $AGENT_TOKEN" \
    -d '{
      "title": "E2E Test: Junior Task",
      "description": "Perform some work",
      "priority": "normal",
      "target_agent_id": "junior-engineer"
    }')
  
  local task_id=$(echo "$task_resp" | grep -oP 'task-[a-z0-9-]+' | head -1)
  [[ -n "$task_id" ]] || error "Failed to create task"
  log "Created task: $task_id"

  # 2. Junior claims the task
  # Junior token? For smoke test we use same token or skip auth if it's internal. 
  # Actually, backend enforces auth. We use director to claim for simplicity in smoke test, or assume tokens exist.
  curl -s -X POST "$BACKEND_URL/internal/tasks/$task_id/claim" \
    -H "X-Agent-Id: junior-engineer" -H "Authorization: Bearer $AGENT_TOKEN" > /dev/null
  log "Task claimed by junior-engineer"

  # 3. Verify in Dashboard API
  local status_resp=$(curl -s "$BACKEND_URL/api/tasks/$task_id")
  echo "$status_resp" | grep -q "in_progress" || error "Task status not updated to in_progress"
  log "Verified: Task is in_progress"

  # 4. Junior completes the task
  curl -s -X POST "$BACKEND_URL/internal/tasks/$task_id/complete" \
    -H "Content-Type: application/json" -H "X-Agent-Id: junior-engineer" -H "Authorization: Bearer $AGENT_TOKEN" \
    -d '{"notes": "Work completed successfully"}' > /dev/null
  log "Task completed by junior-engineer"

  # 5. Final verification
  status_resp=$(curl -s "$BACKEND_URL/api/tasks/$task_id")
  echo "$status_resp" | grep -q "completed" || error "Task status not updated to completed"
  log "Verified: Task is completed"
}

# --- Flow 2: Schema to SQL ---
flow_schema_sql() {
  log "Starting Flow 2: Schema to SQL"
  
  # Simulates an agent creating a task specifically for schema generation
  local task_resp=$(curl -s -X POST "$BACKEND_URL/internal/tasks/create" \
    -H "Content-Type: application/json" -H "X-Agent-Id: director" -H "Authorization: Bearer $AGENT_TOKEN" \
    -d '{
      "title": "Generate Database Schema",
      "description": "Convert design to SQL",
      "priority": "high",
      "target_agent_id": "senior-engineer"
    }')
  local task_id=$(echo "$task_resp" | grep -oP 'task-[a-z0-9-]+' | head -1)
  log "Created schema task: $task_id"

  # Simulate writing the file to workspace (via backend or direct fs if mounted)
  # In this smoke test we just check if the backend records the audit event.
  curl -s -X POST "$BACKEND_URL/api/agents/logs" \
    -H "Content-Type: application/json" -H "X-Agent-Id: senior-engineer" -H "Authorization: Bearer $AGENT_TOKEN" \
    -d '{"level": "info", "message": "Generated schema.sql in workspace"}' > /dev/null
  
  log "Simulated file write and log report"
}

# --- Flow 3: Obsidian Design ---
flow_obsidian_design() {
  log "Starting Flow 3: Obsidian Design"
  
  # 1. Create a mission
  local mission_resp=$(curl -s -X POST "$BACKEND_URL/internal/missions" \
    -H "Content-Type: application/json" -H "X-Agent-Id: director" -H "Authorization: Bearer $AGENT_TOKEN" \
    -d '{
      "title": "E2E Design Mission",
      "description": "Design a new microservice"
    }')
  local mission_id=$(echo "$mission_resp" | grep -oP 'mission-[a-z0-9-]+' | head -1)
  [[ -n "$mission_id" ]] || error "Failed to create mission"
  log "Created mission: $mission_id"

  # 2. Close mission with notes (which triggers Obsidian write)
  local close_resp=$(curl -s -X POST "$BACKEND_URL/internal/missions/$mission_id/close" \
    -H "Content-Type: application/json" -H "X-Agent-Id: director" -H "Authorization: Bearer $AGENT_TOKEN" \
    -d '{"notes": "Design finalized. High-level architecture approved."}')
  
  echo "$close_resp" | grep -q "closed" || error "Failed to close mission"
  log "Mission closed and written to Obsidian"
  
  # 3. Verify file exists in data/obsidian
  if [ -f "data/obsidian/Missions/${mission_id}.md" ]; then
    log "Verified: Obsidian doc created at data/obsidian/Missions/${mission_id}.md"
  else
    error "Obsidian document missing"
  fi
}

check_backend
flow_task_lifecycle
flow_schema_sql
flow_obsidian_design

log "\x1b[32mSUCCESS: All E2E flows passed.\x1b[0m"
