#!/bin/sh
set -u
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
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

run_gate 'WASM type check' ./dev.sh check --locked --no-default-features --features hydrate --lib --target wasm32-unknown-unknown
run_gate 'SSR and WASM build' ./dev.sh leptos build
run_gate 'Rust tests' ./dev.sh test --locked
exit "$status"
