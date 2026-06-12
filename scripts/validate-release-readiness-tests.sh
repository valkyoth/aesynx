#!/usr/bin/env sh
set -eu

root="$(pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

tag="v9.9.9"

cd "$tmpdir"
git init -q
git -c user.name='Aesynx Test' \
    -c user.email='aesynx-test@example.invalid' \
    -c commit.gpgsign=false \
    commit --allow-empty -m 'test release readiness' >/dev/null

head_commit="$(git rev-parse HEAD)"
mkdir -p "security/pentest"
report="security/pentest/$tag.md"

write_report() {
    commit="$1"
    status="$2"
    extra="${3:-}"
    cat >"$report" <<EOF
Tag: $tag
Commit: $commit
Status: $status
Tester: release-readiness self-test
Date: 2026-06-12
Scope: self-test fixture
$extra
EOF
}

write_report "$head_commit" "PASS"
"$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null

write_report "0000000000000000000000000000000000000000" "PASS"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: stale commit hash did not block release" >&2
    exit 1
fi

write_report "$head_commit" "FAIL"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: failed status did not block release" >&2
    exit 1
fi

write_report "$head_commit" "PASS" "TODO: unresolved finding"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: unresolved TODO did not block release" >&2
    exit 1
fi

write_report "$head_commit" "PASS"
git tag "$tag"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: existing local tag did not block release" >&2
    exit 1
fi
git tag -d "$tag" >/dev/null

write_report "$head_commit" "PASS"
cat >PENTEST.md <<'EOF'
temporary findings must block release readiness
EOF

if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: root PENTEST.md did not block release" >&2
    exit 1
fi

echo "release readiness tests: ok"
