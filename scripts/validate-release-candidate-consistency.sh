#!/usr/bin/env sh
set -eu

candidate="$(
    awk '
        /^## Current Candidate$/ { in_current = 1; next }
        /^## / && in_current { exit }
        in_current && match($0, /v[0-9]+\.[0-9]+\.[0-9]+/) {
            print substr($0, RSTART, RLENGTH)
            exit
        }
    ' docs/releases/README.md
)"

if [ -z "$candidate" ]; then
    echo "release candidate consistency: current candidate tag not found" >&2
    exit 1
fi

previous_patch_candidate="$(
    printf '%s\n' "$candidate" |
        awk -F'[v.]' '{ if ($4 > 0) printf "v%s.%s.%s", $2, $3, $4 - 1 }'
)"

release_notes="docs/releases/$candidate-rc.md"
if [ ! -s "$release_notes" ]; then
    echo "release candidate consistency: missing current release notes: $release_notes" >&2
    exit 1
fi

require_contains() {
    file="$1"
    text="$2"
    if ! grep -Fq "$text" "$file"; then
        echo "release candidate consistency: $file missing: $text" >&2
        exit 1
    fi
}

require_not_contains() {
    file="$1"
    text="$2"
    if [ -n "$text" ] && grep -Fq "$text" "$file"; then
        echo "release candidate consistency: $file contains stale candidate text: $text" >&2
        exit 1
    fi
}

require_contains "$release_notes" "# Aesynx $candidate Release Candidate Notes"
require_contains README.md "\`$candidate\` is the current"
require_contains README.md "cargo xtask release-ready $candidate"
require_contains README.md "[$candidate Release Candidate Notes]($release_notes)"

require_contains docs/build-skeleton.md "build/qemu/aesynx-$candidate.iso"
require_contains docs/build-skeleton.md "build/qemu/aesynx-$candidate-panic.iso"
require_contains docs/build-skeleton.md "build/qemu/aesynx-$candidate-exception.iso"
require_contains docs/build-skeleton.md "build/qemu/aesynx-$candidate-timer.iso"

require_contains tools/xtask/src/image/names.rs "aesynx-$candidate.iso"
require_contains tools/xtask/src/image/names.rs "aesynx-$candidate.manifest"
require_contains tools/xtask/src/image/names.rs "aesynx-$candidate.serial.log"
require_contains tools/xtask/src/image/names.rs "aesynx-$candidate-panic.iso"
require_contains tools/xtask/src/image/names.rs "aesynx-$candidate-exception.iso"
require_contains tools/xtask/src/image/names.rs "aesynx-$candidate-timer.iso"

require_contains tools/xtask/src/image/manifest.rs "Aesynx $candidate"
require_contains tools/xtask/src/image/tests.rs "aesynx-$candidate.iso"
require_contains tools/xtask/src/image/tests.rs "Aesynx $candidate"

require_not_contains "$release_notes" "$previous_patch_candidate"
require_not_contains docs/build-skeleton.md "$previous_patch_candidate"
require_not_contains tools/xtask/src/image/names.rs "$previous_patch_candidate"
require_not_contains tools/xtask/src/image/manifest.rs "$previous_patch_candidate"
require_not_contains tools/xtask/src/image/tests.rs "$previous_patch_candidate"

echo "release candidate consistency: ok for $candidate"
