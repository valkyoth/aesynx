#!/usr/bin/env sh
set -eu

echo "checks: repository security policy"
scripts/validate-security-policy.sh

echo "checks: release readiness policy"
scripts/validate-release-readiness-tests.sh

echo "checks: release candidate consistency"
scripts/validate-release-candidate-consistency.sh

echo "checks: documentation links"
perl scripts/check-doc-links.pl

echo "checks: modularity policy"
scripts/validate-modularity-policy.sh

echo "checks: kernel engineering policy"
scripts/validate-kernel-policy.sh

if [ -f Cargo.toml ] && ! grep -q '^members = \[\]$' Cargo.toml; then
    echo "checks: cargo metadata"
    cargo metadata --format-version 1 >/dev/null

    echo "checks: formatting"
    cargo fmt --all --check

    echo "checks: workspace check"
    cargo check --workspace

    echo "checks: clippy"
    cargo clippy --workspace --all-targets -- -D warnings

    echo "checks: panic smoke feature clippy"
    cargo clippy -p aesynx-kernel --features panic-smoke --target x86_64-unknown-none -- -D warnings

    echo "checks: exception smoke feature clippy"
    cargo clippy -p aesynx-kernel --features exception-smoke --target x86_64-unknown-none -- -D warnings

    echo "checks: timer smoke feature clippy"
    cargo clippy -p aesynx-kernel --features timer-smoke --target x86_64-unknown-none -- -D warnings

    echo "checks: tests"
    cargo test --workspace

    echo "checks: fuzz/property smoke"
    cargo xtask fuzz-smoke

    echo "checks: dependency policy"
    if cargo deny --version >/dev/null 2>&1; then
        cargo deny check
    else
        echo "checks: cargo deny not installed; skipping local dependency policy check" >&2
    fi

    if [ -f Cargo.lock ]; then
        echo "checks: RustSec advisories"
        if cargo audit --version >/dev/null 2>&1; then
            cargo audit
        else
            echo "checks: cargo audit not installed; skipping local advisory check" >&2
        fi
    else
        echo "checks: Cargo.lock not present yet; skipping cargo audit until crates land"
    fi
fi

if [ -f Cargo.toml ] && grep -q '^members = \[\]$' Cargo.toml; then
    echo "checks: empty Rust workspace; cargo checks start when the first crate is added"
fi

echo "checks: ok"
