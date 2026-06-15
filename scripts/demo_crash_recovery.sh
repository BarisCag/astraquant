#!/usr/bin/env bash
# Demonstrates journal persistence and replay hash stability (Linux/macOS).
set -euo pipefail

export ASTRA_NODE_ID="demo-node"
export ASTRA_HTTP_PORT=8081
export ASTRA_JOURNAL_PATH="journal_demo.astra_jl"
rm -f $ASTRA_JOURNAL_PATH

echo "Seeding journal..."
cargo run --release -p astra-ops &
PID=$!
sleep 3

curl -s -X POST http://127.0.0.1:8081/ingest -d '{"dummy":2}'
sleep 1
PRE_HASH=$(curl -s http://127.0.0.1:8081/metrics | grep "astra_kernel_state_hash" | awk '{print $2}')
echo "Pre-crash hash: $PRE_HASH"

kill -9 "$PID" 2>/dev/null || true
wait "$PID" 2>/dev/null || true

echo "Recovering via replay..."
cargo run --release -p astra-ops &
PID2=$!
sleep 3

POST_HASH=$(curl -s http://127.0.0.1:8081/metrics | grep "astra_kernel_state_hash" | awk '{print $2}')
echo "Post-crash hash: $POST_HASH"

kill -9 "$PID2" 2>/dev/null || true
wait "$PID2" 2>/dev/null || true

rm -f $ASTRA_JOURNAL_PATH

if [ "$PRE_HASH" != "$POST_HASH" ] || [ -z "$PRE_HASH" ]; then
    echo "ERROR: Hashes do not match or are empty!"
    exit 1
fi

echo "Inspect logs for identical state_hash lines after recovery. SUCCESS!"
