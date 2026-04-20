#!/usr/bin/env bash
# Sends a mock heartbeat to the backend to verify Live Telemetry in the dashboard.
set -euo pipefail

BACKEND_URL="http://127.0.0.1:8080"
AGENT_ID="director"
AGENT_TOKEN="director-token" # Default from .env.example

# Try to get real token from .env if it exists
if [ -f .env ]; then
  # Extract the token for 'director' from AGENT_TOKENS=director=token1,contractor=token2
  TOKEN=$(grep "^AGENT_TOKENS=" .env | cut -d'=' -f2- | tr ',' '\n' | grep "^${AGENT_ID}=" | cut -d'=' -f2 || echo "")
  if [ -n "$TOKEN" ]; then
    AGENT_TOKEN="$TOKEN"
  fi
fi

echo "Sending heartbeat for agent '$AGENT_ID' to $BACKEND_URL..."

curl -X POST "$BACKEND_URL/api/agents/heartbeat" \
  -H "Content-Type: application/json" \
  -H "x-agent-id: $AGENT_ID" \
  -H "Authorization: Bearer $AGENT_TOKEN" \
  -d '{
    "status": "working",
    "current_task_id": "mock-task-123"
  }'

echo -e "\nHeartbeat sent! Check the 'Intelligence' page in the dashboard."
