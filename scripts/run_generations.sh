#!/bin/bash
#
# Run datasynth generation with config and output directory.
#
# Usage:
#   ./scripts/run_generations.sh                    # config.yaml -> ./output
#   ./scripts/run_generations.sh --demo             # Demo preset -> ./output
#   ./scripts/run_generations.sh -c my.yaml -o out  # Custom config and output
#   ./scripts/run_generations.sh --validate         # Validate config only
#

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

BIN="${REPO_ROOT}/target/release/datasynth-data"
CONFIG="${REPO_ROOT}/config.yaml"
OUTPUT="${REPO_ROOT}/output"
DEMO=false
VALIDATE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -c|--config) CONFIG="$2"; shift 2 ;;
        -o|--output) OUTPUT="$2"; shift 2 ;;
        --demo)      DEMO=true; shift ;;
        --validate)  VALIDATE=true; shift ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo "  -c, --config PATH  Config file (default: config.yaml)"
            echo "  -o, --output PATH  Output directory (default: ./output)"
            echo "  --demo              Use demo preset (ignores -c)"
            echo "  --validate          Validate config and exit"
            exit 0
            ;;
        *) shift ;;
    esac
done

if [[ ! -x "$BIN" ]]; then
    echo "Binary not found. Run: ./scripts/build_crates.sh"
    exit 1
fi

if "$VALIDATE"; then
    "$BIN" validate --config "$CONFIG"
    exit $?
fi

if "$DEMO"; then
    "$BIN" generate --demo --output "$OUTPUT"
else
    if [[ ! -f "$CONFIG" ]]; then
        echo "Config not found: $CONFIG"
        echo "Create one with: $BIN init --industry manufacturing --complexity medium -o $CONFIG"
        exit 1
    fi
    "$BIN" generate --config "$CONFIG" --output "$OUTPUT"
fi

echo "Generation complete. Output: $OUTPUT"
