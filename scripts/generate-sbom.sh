#!/usr/bin/env sh
set -eu

mkdir -p sbom

if [ ! -f Cargo.toml ]; then
    echo "sbom: no Cargo.toml present"
    exit 0
fi

if grep -q '^members = \[\]$' Cargo.toml; then
    echo "sbom: empty Rust workspace; skipping until the first crate is added"
    exit 0
fi

if ! cargo sbom --version >/dev/null 2>&1; then
    echo "sbom: cargo sbom not installed; install with: cargo install --locked cargo-sbom --version 0.10.0" >&2
    exit 1
fi

cargo sbom --output-format spdx_json_2_3 > sbom/aesynx.spdx.json
test -s sbom/aesynx.spdx.json

echo "sbom: wrote sbom/aesynx.spdx.json"
