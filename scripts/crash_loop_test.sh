#!/bin/bash
set -e

echo "Starting AstraDaemon Crash-Loop Recovery Harness..."

# 1. Start daemon in background
export ASTRA_NODE_ID="crash-test"
export ASTRA_HTTP_PORT=8080
export ASTRA_JOURNAL_DIR="journal_crash_test.astra_jl"
rm -f $ASTRA_JOURNAL_DIR

cargo run --release -p astra-ops &
DAEMON_PID=$!

echo "Daemon started with PID $DAEMON_PID. Injecting event..."
sleep 3 # wait for boot

curl -s -X POST http://127.0.0.1:8080/ingest -d '{"dummy":1}'
sleep 1

PRE_CRASH_HASH=$(curl -s http://127.0.0.1:8080/metrics | grep "astra_kernel_state_hash" | awk '{print $2}')
echo "Pre-crash hash: $PRE_CRASH_HASH"

# 2. Force crash
echo "Simulating sudden OS failure (SIGKILL)..."
kill -9 $DAEMON_PID
wait $DAEMON_PID 2>/dev/null || true

# 3. Boot recovery
echo "Restarting daemon from journal..."
cargo run --release -p astra-ops &
DAEMON_PID2=$!

sleep 3

POST_CRASH_HASH=$(curl -s http://127.0.0.1:8080/metrics | grep "astra_kernel_state_hash" | awk '{print $2}')
echo "Post-crash hash: $POST_CRASH_HASH"

kill -9 $DAEMON_PID2
wait $DAEMON_PID2 2>/dev/null || true
rm -f $ASTRA_JOURNAL_DIR

if [ "$PRE_CRASH_HASH" != "$POST_CRASH_HASH" ] || [ -z "$PRE_CRASH_HASH" ]; then
    echo "ERROR: Hashes do not match or are empty!"
    exit 1
fi

echo "Crash-Loop Recovery Test SUCCESSFUL: Deterministic Replay matches expected state."
