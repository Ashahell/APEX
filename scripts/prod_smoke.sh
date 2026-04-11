#!/usr/bin/env bash
set -euo pipefail

HOST="${1:-http://localhost}"
# Expanded endpoints for prod health checks
ENDPOINTS=(
  "/health" "/status" "/api/v1/heartbeat/stats" 
  "/api/v1/system/health" "/api/v1/metrics" "/api/v1/memory/stats" 
  "/api/v1/memory/index" "/api/v1/memory/bounded/stats" 
  "/api/v1/journal" "/api/v1/channels" "/api/v1/events" 
)
HOSTS=("${HOST}:3000" "${HOST}:3001")
for ep in "${ENDPOINTS[@]}"; do
  reachable=false
  for h in "${HOSTS[@]}"; do
    if curl -sf "$h$ep" >/dev/null 2>&1; then
      echo "OK: $ep on $h" 
      reachable=true
      break
    fi
  done
  if [ "$reachable" = false ]; then
    echo "ERR: $ep not reachable on any known host/port" >&2; exit 1
  fi
done
echo "Prod smoke test passed."
