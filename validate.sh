#!/bin/sh
set -u
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT" || exit 1
status=0

run_gate() {
  label=$1
  shift
  printf '\n+++ %s +++\n' "$label"
  if "$@"; then
    printf 'PASS %s\n' "$label"
  else
    printf 'FAIL %s\n' "$label" >&2
    status=1
  fi
}

# Invoked indirectly through run_gate.
# shellcheck disable=SC2329
check_static_export() {
  ./export.sh || return 1
  for page in index.html 404.html writing/index.html; do
    if [ ! -s "run/pages/$page" ]; then
      printf 'Missing or empty exported page: run/pages/%s\n' "$page" >&2
      return 1
    fi
  done
}

run_gate 'WASM type check' ./dev.sh check --locked --no-default-features --features hydrate --lib --target wasm32-unknown-unknown
run_gate 'SSR and WASM build' ./dev.sh leptos build
run_gate 'Rust tests' ./dev.sh test --locked
run_gate 'Static Pages export' check_static_export
exit "$status"
