#!/usr/bin/env sh
set -eu

if [ "${1:-}" = "" ]; then
    echo "release readiness: usage: scripts/validate-release-readiness.sh vX.Y.Z" >&2
    exit 2
fi

tag="$1"

if ! printf '%s' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "release readiness: tag must look like vX.Y.Z: $tag" >&2
    exit 2
fi

if [ -e PENTEST.md ]; then
    echo "release readiness: root PENTEST.md is still present; resolve and delete it before release" >&2
    exit 1
fi

if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    echo "release readiness: tag already exists locally: $tag" >&2
    exit 1
fi

report="security/pentest/$tag.md"

if [ ! -s "$report" ]; then
    echo "release readiness: missing pentest report: $report" >&2
    exit 1
fi

if [ -f Cargo.toml ] && ! grep -q '^members = \[\]$' Cargo.toml; then
    if [ ! -s sbom/aesynx.spdx.json ]; then
        echo "release readiness: missing generated SBOM: sbom/aesynx.spdx.json" >&2
        exit 1
    fi
fi

head_commit="$(git rev-parse HEAD)"

if ! grep -Fxq "Tag: $tag" "$report"; then
    echo "release readiness: report must contain exact tag line: Tag: $tag" >&2
    exit 1
fi

if ! grep -Fxq "Commit: $head_commit" "$report"; then
    echo "release readiness: report must target current HEAD: $head_commit" >&2
    exit 1
fi

if ! grep -q '^Status: PASS$' "$report"; then
    echo "release readiness: report must contain Status: PASS" >&2
    exit 1
fi

if ! grep -Eq '^Tester: .*[^[:space:]].*$' "$report"; then
    echo "release readiness: report must contain non-blank Tester field" >&2
    exit 1
fi

if ! grep -Eq '^Date: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$report"; then
    echo "release readiness: report must contain Date: YYYY-MM-DD" >&2
    exit 1
fi

if ! grep -Eq '^Scope: .*[^[:space:]].*$' "$report"; then
    echo "release readiness: report must contain non-blank Scope field" >&2
    exit 1
fi

if grep -qE 'TODO|TBD|Status: FAIL|Status: BLOCKED' "$report"; then
    echo "release readiness: report contains unresolved status text" >&2
    exit 1
fi

echo "release readiness: ok for $tag"
