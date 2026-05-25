#!/usr/bin/env bash
# Demonstrates journal persistence and replay hash stability (Linux/macOS).
set -euo pipefail

JOURNAL_DIR="${ASTRA_JOURNAL_DIR:-/tmp/astra_showcase}"
rm -rf "$JOURNAL_DIR"
mkdir -p "$JOURNAL_DIR"
export ASTRA_JOURNAL_DIR="$JOURNAL_DIR"

echo "Seeding journal..."
cargo run --release -p astra-ops &
PID=$!
sleep 2
PRE_HASH=$(grep -E 'state_hash=' "$JOURNAL_DIR/../astra_live.log" 2>/dev/null || true)
kill "$PID" 2>/dev/null || true
wait "$PID" 2>/dev/null || true

echo "Recovering via replay..."
cargo run --release -p astra-ops 2>&1 | tee /tmp/astra_recovery.log &
PID=$!
sleep 2
kill "$PID" 2>/dev/null || true

echo "Inspect logs for identical state_hash lines after recovery."
