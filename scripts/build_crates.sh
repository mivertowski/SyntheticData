#!/bin/bash
#
# Build datasynth crates (release binary).
#
# Usage:
#   ./scripts/build_crates.sh           # Release build
#   ./scripts/build_crates.sh --check   # cargo check only
#   ./scripts/build_crates.sh --test    # Run tests then build
#   ./scripts/build_crates.sh --fmt    # Format + clippy then build
#

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

CHECK=false
TEST=false
FMT=false

for arg in "$@"; do
    case "$arg" in
        --check) CHECK=true ;;
        --test)  TEST=true ;;
        --fmt)   FMT=true ;;
        --help|-h)
            echo "Usage: $0 [--check|--test|--fmt]"
            echo "  --check  Run cargo check only"
            echo "  --test   Run cargo test then build"
            echo "  --fmt    Run cargo fmt and clippy then build"
            exit 0
            ;;
    esac
done

if "$FMT"; then
    cargo fmt
    cargo clippy --all-targets -- -D warnings
fi

if "$TEST"; then
    cargo test
fi


JOBS=$(($(nproc) - 2))
if "$CHECK"; then
    cargo check --release -j$JOBS
else
    cargo build --workspace --all-targets --release -j$JOBS
fi


echo "Build complete. Binary: $REPO_ROOT/target/release/datasynth-data"
