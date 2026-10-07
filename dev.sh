#!/bin/sh
set -eu

ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
case "${BLOG_STATIC_BUILD-0}" in
  0)
    SITE_ROOT=site
    HASH_FILES=false
    ;;
  1)
    SITE_ROOT=site-release
    HASH_FILES=true
    ;;
  *)
    printf '%s\n' 'BLOG_STATIC_BUILD must be 0 or 1' >&2
    exit 2
    ;;
esac
TOOLCHAIN=1.99.0
mkdir -p "$ROOT/run/cargo" "$ROOT/run/build" "$ROOT/run/cache" "$ROOT/run/tmp" \
  "$ROOT/run/trivy" "$ROOT/run/rustup" "$ROOT/run/data" "$ROOT/run/config"
export CARGO_HOME="$ROOT/run/cargo"
export CARGO_TARGET_DIR="$ROOT/run/build"
export XDG_CACHE_HOME="$ROOT/run/cache"
export XDG_DATA_HOME="$ROOT/run/data"
export XDG_CONFIG_HOME="$ROOT/run/config"
export TMPDIR="$ROOT/run/tmp"
export TRIVY_CACHE_DIR="$ROOT/run/trivy"
export RUSTUP_HOME="$ROOT/run/rustup"
export RUSTUP_TOOLCHAIN="$TOOLCHAIN"
export RUSTUP_AUTO_INSTALL=0

# Process environment takes precedence over cargo-leptos' parent-directory .env lookup.
export LEPTOS_SITE_ROOT="$ROOT/run/$SITE_ROOT"
export LEPTOS_SITE_PKG_DIR=pkg
export LEPTOS_ASSETS_DIR="$ROOT/public"
export LEPTOS_STYLE_FILE="$ROOT/assets/site.css"
export LEPTOS_BIN_TARGET_DIR="$ROOT/run/build"
export LEPTOS_OUTPUT_NAME=mcb-smart-boy
export LEPTOS_HASH_FILES="$HASH_FILES"
cd "$ROOT"


if [ "${1:-}" = setup ]; then
  rustup toolchain install "$TOOLCHAIN" --profile minimal \
    --component rustfmt --component clippy --target wasm32-unknown-unknown --no-self-update
else
  if ! rustup toolchain list | grep -F "$TOOLCHAIN" >/dev/null; then
    printf '%s\n' "Project Rust toolchain missing; run ./dev.sh setup" >&2
    exit 2
  fi
fi

LOCAL_CARGO=$(rustup which --toolchain "$TOOLCHAIN" cargo)
LOCAL_CARGO_BIN=$(dirname -- "$LOCAL_CARGO")
export PATH="$CARGO_HOME/bin:$LOCAL_CARGO_BIN:$PATH"
export LEPTOS_BIN_CARGO_COMMAND="$LOCAL_CARGO"
if [ "${1:-}" = setup ]; then
  if [ ! -x "$CARGO_HOME/bin/cargo-leptos" ] ||
    ! "$CARGO_HOME/bin/cargo-leptos" --version | grep -F '0.3.10' >/dev/null; then
    cargo install cargo-leptos --version 0.3.10 --locked --features no_downloads --root "$CARGO_HOME"
  fi
  if [ ! -x "$CARGO_HOME/bin/wasm-bindgen" ] ||
    ! "$CARGO_HOME/bin/wasm-bindgen" --version | grep -F '0.2.129' >/dev/null; then
    cargo install wasm-bindgen-cli --version 0.2.129 --locked --root "$CARGO_HOME"
  fi
  if [ ! -x "$CARGO_HOME/bin/wasm-opt" ] ||
    ! cargo install --list --root "$CARGO_HOME" | grep -Fx 'wasm-opt v0.116.1:' >/dev/null; then
    # Release WASM has no DWARF; omit the optional LLVM debug-info passes.
    cargo install wasm-opt --version 0.116.1 --locked --no-default-features --root "$CARGO_HOME"
  fi
  exit 0
fi
if [ "${1:-}" = leptos ] && [ ! -x "$CARGO_HOME/bin/cargo-leptos" ]; then
  printf '%s\n' "Project cargo-leptos missing; run ./dev.sh setup" >&2
  exit 2
fi
if [ "${1:-}" = -- ]; then
  shift
  if [ "$#" -eq 0 ]; then
    printf '%s\n' "usage: ./dev.sh -- command [arguments...]" >&2
    exit 2
  fi
  exec "$@"
fi
exec cargo "$@"
