#!/usr/bin/env sh
set -eu

if [ "${1:-}" = "" ]; then
    echo "rustc workspace wrapper: missing rustc path" >&2
    exit 2
fi

rustc="$1"
shift

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
workspace_root="$(CDPATH= cd -- "$script_dir/.." && pwd)"

exec "$rustc" --remap-path-prefix "$workspace_root=." "$@"
