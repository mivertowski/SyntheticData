#!/usr/bin/env bash
# Safe workspace test runner — runs crates sequentially with limited parallelism
# to prevent OOM crashes from 400MB+ debug test binaries.
#
# Usage:
#   ./scripts/test-safe.sh            # Run all crates
#   ./scripts/test-safe.sh --quick    # Skip heavy crates (server, eval, runtime)
#   ./scripts/test-safe.sh -p <crate> # Run a single crate
set -euo pipefail

# --- Configuration -----------------------------------------------------------
MAX_TEST_THREADS="${TEST_THREADS:-2}"
QUICK_MODE=false
SINGLE_CRATE=""

# Light crates: fast compilation, small test binaries
LIGHT_CRATES=(
    datasynth-core
    datasynth-test-utils
    datasynth-config
    datasynth-output
    datasynth-standards
    datasynth-banking
    datasynth-ocpm
    datasynth-fingerprint
    datasynth-graph
)

# Heavy crates: large binaries, data-generating tests
HEAVY_CRATES=(
    datasynth-generators
    datasynth-eval
    datasynth-runtime
    datasynth-cli
)

# Excluded by default (requires protoc / external services)
EXCLUDED_CRATES=(
    datasynth-server
    datasynth-graph-export
)

# --- Argument parsing ---------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --quick|-q)  QUICK_MODE=true; shift ;;
        -p)          SINGLE_CRATE="$2"; shift 2 ;;
        -t)          MAX_TEST_THREADS="$2"; shift 2 ;;
        --help|-h)
            echo "Usage: $0 [--quick] [-p <crate>] [-t <threads>]"
            echo "  --quick    Skip heavy crates (generators, eval, runtime, cli)"
            echo "  -p <crate> Run a single crate"
            echo "  -t <n>     Max test threads (default: 2)"
            exit 0
            ;;
        *)           echo "Unknown option: $1"; exit 1 ;;
    esac
done

# --- Helpers ------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
PASS=0
FAIL=0
SKIP=0
# Helper to safely increment counters under set -e
inc() { eval "$1=\$(( $1 + 1 ))"; }

run_crate() {
    local crate="$1"
    printf "${YELLOW}[test]${NC} %-30s" "$crate"
    local output
    output=$(cargo test -p "$crate" -- --test-threads="$MAX_TEST_THREADS" 2>&1)
    local exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
        printf " ${GREEN}PASS${NC}\n"
        inc PASS
    else
        printf " ${RED}FAIL${NC}\n"
        echo "$output" | tail -20
        inc FAIL
    fi
}

# --- Main ---------------------------------------------------------------------
echo "=== DataSynth Safe Test Runner ==="
echo "    Threads per crate: $MAX_TEST_THREADS"
echo ""

if [[ -n "$SINGLE_CRATE" ]]; then
    run_crate "$SINGLE_CRATE"
else
    echo "--- Light crates ---"
    for crate in "${LIGHT_CRATES[@]}"; do
        run_crate "$crate"
    done

    if [[ "$QUICK_MODE" == false ]]; then
        echo ""
        echo "--- Heavy crates (sequential, low parallelism) ---"
        for crate in "${HEAVY_CRATES[@]}"; do
            run_crate "$crate"
        done
    else
        SKIP=$((SKIP + ${#HEAVY_CRATES[@]}))
        echo ""
        echo "--- Skipped ${#HEAVY_CRATES[@]} heavy crates (--quick mode) ---"
    fi

    echo ""
    echo "--- Excluded crates (need protoc/external deps) ---"
    for crate in "${EXCLUDED_CRATES[@]}"; do
        printf "${YELLOW}[skip]${NC} %-30s\n" "$crate"
        inc SKIP
    done
fi

echo ""
echo "=== Results: ${GREEN}${PASS} passed${NC}, ${RED}${FAIL} failed${NC}, ${YELLOW}${SKIP} skipped${NC} ==="

if [[ $FAIL -gt 0 ]]; then
    exit 1
fi
