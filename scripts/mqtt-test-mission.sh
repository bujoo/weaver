#!/bin/bash
# Publish a test mission to EMQX simulating what Brain would send.
# Prerequisites: EMQX running on localhost:1883, mqttx installed.
#
# Usage:
#   ./scripts/mqtt-test-mission.sh           # Publish all state (retained)
#   ./scripts/mqtt-test-mission.sh assign    # Also publish P0 assignment

set -euo pipefail

HOST="localhost"
PORT="1883"
WS="dev"
MID="68206c25-233b-4214-b6b1-0e94f7ee9234"
DIR="/tmp/weaver-mqtt-test"
USERNAME="andre-mac"
PASSWORD="weaver-dev-secret"

echo "=== Publishing test mission to EMQX ==="
echo "Host: $HOST:$PORT | Workspace: $WS | Mission: ${MID:0:8}..."
echo ""

# 1. Registry (retained) - triggers workspace auto-setup
echo "[1/4] Publishing registry..."
mqttx pub -h "$HOST" -p "$PORT" -u "$USERNAME" -P "$PASSWORD" \
  -t "weaver/$WS/registry" --retain \
  -m "$(cat $DIR/registry.json)" 2>/dev/null
echo "  -> weaver/$WS/registry"

# 2. Plan state (retained) - full plan
echo "[2/4] Publishing plan state..."
mqttx pub -h "$HOST" -p "$PORT" -u "$USERNAME" -P "$PASSWORD" \
  -t "weaver/$WS/state/$MID/plan" --retain \
  -m "$(cat $DIR/plan-state.json)" 2>/dev/null
echo "  -> weaver/$WS/state/$MID/plan"

# 3. Phase states (retained)
echo "[3/4] Publishing phase states..."
for f in $DIR/phase-P*.json; do
  PID=$(basename "$f" .json | sed 's/phase-//')
  mqttx pub -h "$HOST" -p "$PORT" -u "$USERNAME" -P "$PASSWORD" \
    -t "weaver/$WS/state/$MID/phase/$PID" --retain \
    -m "$(cat $f)" 2>/dev/null
  echo "  -> weaver/$WS/state/$MID/phase/$PID"
done

# 4. Todo states (retained)
echo "[4/4] Publishing todo states..."
COUNT=0
for f in $DIR/todo-P*.json; do
  TID=$(basename "$f" .json | sed 's/todo-//')
  mqttx pub -h "$HOST" -p "$PORT" -u "$USERNAME" -P "$PASSWORD" \
    -t "weaver/$WS/state/$MID/todo/$TID" --retain \
    -m "$(cat $f)" 2>/dev/null
  COUNT=$((COUNT + 1))
done
echo "  -> $COUNT todo states published"

echo ""
echo "=== State published (all retained) ==="
echo "Weaver should now cache: 1 plan, 4 phases, $COUNT todos"

# Optional: publish assignment for P0
if [[ "${1:-}" == "assign" ]]; then
  echo ""
  echo "=== Publishing P0 assignment ==="
  mqttx pub -h "$HOST" -p "$PORT" -u "$USERNAME" -P "$PASSWORD" \
    -t "weaver/$WS/assign/pool" \
    -m "$(cat $DIR/assignment-P0.json)" 2>/dev/null
  echo "  -> weaver/$WS/assign/pool (P0 with 7 todos)"
  echo ""
  echo "Weaver should now auto-execute Phase P0!"
fi
