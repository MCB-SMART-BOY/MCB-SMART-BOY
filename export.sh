#!/bin/sh
set -eu

ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
export BLOG_STATIC_BUILD=1

"$ROOT/dev.sh" leptos build --release
exec "$ROOT/dev.sh" -- "$ROOT/run/build/release/mcb-smart-boy" export
