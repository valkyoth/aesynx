#!/usr/bin/env sh
set -eu

if [ ! -s docs/modularity-policy.md ]; then
    echo "modularity policy: docs/modularity-policy.md missing" >&2
    exit 1
fi

if [ -d src ]; then
    echo "modularity policy: root src/ is not allowed; use crates/ and models/" >&2
    exit 1
fi

for file in $(find crates models tools tests -type f -name '*.rs' 2>/dev/null || true); do
    case "$file" in
        */target/*|*/generated/*|*/vendor/*)
            continue
            ;;
    esac
    lines="$(wc -l < "$file" | tr -d ' ')"
    if [ "$lines" -gt 500 ] && ! grep -q "Path: $file" docs/modularity-policy.md; then
        echo "modularity policy: $file has $lines lines; split it or document a temporary exception" >&2
        exit 1
    fi
done

if [ -f Cargo.toml ] && ! grep -q '^resolver = "3"$' Cargo.toml; then
    echo "modularity policy: workspace resolver must be 3 for edition 2024 workspaces" >&2
    exit 1
fi

echo "modularity policy: ok"
