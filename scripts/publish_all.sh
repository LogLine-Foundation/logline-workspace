#!/bin/bash
# LogLine Workspace — crates.io publish script
# 
# Publishes all crates in correct DAG order.
# Run: ./scripts/publish_all.sh
# 
# Pre-requisites:
#   1. cargo login <your-token>
#   2. All tests passing: cargo test --workspace
#   3. Clean git state or use --allow-dirty

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_DIR="$(dirname "$SCRIPT_DIR")"
cd "$WORKSPACE_DIR"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Sleep between publishes to let crates.io index update
DELAY=${DELAY:-30}
DRY_RUN=${DRY_RUN:-false}
ALLOW_DIRTY=${ALLOW_DIRTY:-false}

# Publish order (DAG topological sort)
CRATES=(
  "logline-core"
  "atomic-types"
  "atomic-crypto"
  "json_atomic"
  "atomic-codec"
  "tdln-ast"
  "tdln-proof"
  "lllv-core"
  "tdln-compiler"
  "atomic-sirp"
  "atomic-ubl"
  "lllv-index"
  "tdln-gate"
  "atomic-runtime"
  "logline"
)

publish_crate() {
  local crate=$1
  local extra_args=""
  
  if [ "$ALLOW_DIRTY" = "true" ]; then
    extra_args="--allow-dirty"
  fi
  
  if [ "$DRY_RUN" = "true" ]; then
    extra_args="$extra_args --dry-run"
  fi
  
  echo -e "${YELLOW}Publishing $crate...${NC}"
  
  if cargo publish -p "$crate" $extra_args; then
    echo -e "${GREEN}✓ $crate published successfully${NC}"
    return 0
  else
    # Check if already exists (not an error)
    if cargo publish -p "$crate" $extra_args 2>&1 | grep -q "already exists"; then
      echo -e "${GREEN}✓ $crate already exists on crates.io${NC}"
      return 0
    fi
    echo -e "${RED}✗ Failed to publish $crate${NC}"
    return 1
  fi
}

echo "═══════════════════════════════════════════════════════════"
echo "  LogLine Workspace — crates.io Publish"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "  Dry run:      $DRY_RUN"
echo "  Allow dirty:  $ALLOW_DIRTY"
echo "  Delay:        ${DELAY}s between publishes"
echo ""
echo "  To customize: DRY_RUN=true ALLOW_DIRTY=true DELAY=60 $0"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""

# Pre-flight checks
echo "Running pre-flight checks..."
cargo check --workspace
echo -e "${GREEN}✓ Workspace compiles${NC}"

if [ "$DRY_RUN" != "true" ]; then
  echo ""
  echo "Publishing ${#CRATES[@]} crates in DAG order..."
  echo ""
fi

for crate in "${CRATES[@]}"; do
  publish_crate "$crate"
  
  if [ "$DRY_RUN" != "true" ] && [ "$crate" != "${CRATES[-1]}" ]; then
    echo "  Waiting ${DELAY}s for crates.io index..."
    sleep "$DELAY"
  fi
done

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}  ✓ All crates processed${NC}"
echo "═══════════════════════════════════════════════════════════"
