#!/usr/bin/env sh
set -eu

for manifest in crates/*/Cargo.toml; do
    crate_dir="${manifest%/Cargo.toml}"
    root="$crate_dir/src/lib.rs"

    if [ ! -f "$root" ]; then
        echo "kernel policy: crate root missing: $root" >&2
        exit 1
    fi

    if ! grep -Fq '#![no_std]' "$root"; then
        echo "kernel policy: missing #![no_std]: $root" >&2
        exit 1
    fi

    if ! grep -Eq '#!\[(deny|forbid)\(unsafe_code\)\]' "$root"; then
        echo "kernel policy: missing unsafe-code denial: $root" >&2
        exit 1
    fi

    case "$crate_dir" in
        crates/aesynx-arch-aarch64|crates/aesynx-arch-x86_64)
            if ! grep -Fq '#![deny(unsafe_code)]' "$root"; then
                echo "kernel policy: unsafe-bearing crate must use deny plus documented local exceptions: $root" >&2
                exit 1
            fi
            ;;
        *)
            if ! grep -Fq '#![forbid(unsafe_code)]' "$root"; then
                echo "kernel policy: unsafe-free crate must use #![forbid(unsafe_code)]: $root" >&2
                exit 1
            fi
            ;;
    esac
done

if grep -RInE '(^|[^A-Za-z0-9_])std::|extern crate std' crates --include='*.rs' 2>/dev/null; then
    echo "kernel policy: std usage found under crates/" >&2
    exit 1
fi

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

awk '
    /^\[/ {
        in_deps = ($0 ~ /^\[(workspace\.)?(dependencies|dev-dependencies|build-dependencies)\]$/) ||
                  ($0 ~ /^\[target\..*\.dependencies\]$/) ||
                  ($0 ~ /^\[target\..*\.dev-dependencies\]$/) ||
                  ($0 ~ /^\[target\..*\.build-dependencies\]$/)
        next
    }
    in_deps && /^[[:space:]]*["]?[A-Za-z0-9_-]+["]?[[:space:]]*=/ {
        dep = $1
        gsub(/"/, "", dep)
        gsub(/[[:space:]]*=/, "", dep)
        if (dep !~ /^aesynx-/) {
            print dep
        }
    }
' Cargo.toml crates/*/Cargo.toml tools/*/Cargo.toml | sort -u > "$tmp_file"

while IFS= read -r dep; do
    [ "$dep" = "" ] && continue
    if ! grep -Fq "Crate: $dep" docs/kernel-engineering-policy.md; then
        echo "kernel policy: external dependency lacks exception: $dep" >&2
        exit 1
    fi
done < "$tmp_file"

echo "kernel policy: ok"
