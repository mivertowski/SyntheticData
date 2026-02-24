#!/usr/bin/env bash
# Launch DataSynth Output Viewer (React + Vite) with data from output directory.
# Usage: ./scripts/launch_output_viewer.sh [OUTPUT_DIR]
#   OUTPUT_DIR defaults to ./output if not set.

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${1:-$REPO_ROOT/output}"
VIEWER_DIR="$REPO_ROOT/datasynth-output-viewer"

cd "$VIEWER_DIR"
if [[ ! -d node_modules ]]; then
  npm install
fi
OUTPUT_DIR="$OUTPUT_DIR" npm run dev:with-data
