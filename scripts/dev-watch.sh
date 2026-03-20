#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "cargo-watch ist noch nicht installiert."
  echo "Bitte einmal ausfuehren: cargo install cargo-watch"
  exit 1
fi

export AUTO_RELOAD_ENABLED="${AUTO_RELOAD_ENABLED:-true}"
export AUTO_RELOAD_INTERVAL_MS="${AUTO_RELOAD_INTERVAL_MS:-1200}"

echo "Starte Entwicklungsmodus mit automatischem Rebuild und Browser-Refresh..."

exec cargo watch \
  -w src \
  -w templates \
  -w static \
  -w migrations \
  -w .env \
  -x run
