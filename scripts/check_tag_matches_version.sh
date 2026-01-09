#!/usr/bin/env bash
set -euo pipefail
CRATENAME="${1:?crate-name}"
EXPECT="${2:?version}"

MF=$(cargo metadata --no-deps --format-version=1 | jq -r \
  --arg n "$CRATENAME" '.packages[] | select(.name==$n) | .manifest_path' | head -n1)

test -n "$MF" || { echo "crate not found: $CRATENAME"; exit 1; }

FOUND=$(tomlq -f "$MF" -r '.package.version' 2>/dev/null || true)
if [ -z "$FOUND" ]; then
  FOUND=$(grep -E '^\s*version\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"' -m1 "$MF" | sed -E 's/.*"([^"]+)".*/\1/')
fi

echo "Tag version: $EXPECT ; Cargo.toml version: $FOUND"
test "$FOUND" = "$EXPECT" || { echo "⛔ version mismatch"; exit 1; }
