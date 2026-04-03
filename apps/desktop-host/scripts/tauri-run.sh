#!/usr/bin/env bash
# Tauri CLI зовёт `cargo metadata`; в GUI/IDE и в некоторых терминалах PATH без ~/.cargo/bin.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
fi
export PATH="${HOME}/.cargo/bin:${PATH}"
exec "${ROOT}/node_modules/.bin/tauri" "$@"
