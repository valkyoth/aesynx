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
release_notes="docs/releases/$tag-rc.md"

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

write_release_notes() {
    mkdir -p "$(dirname "$release_notes")"
    cat >"$release_notes" <<EOF
# Aesynx $tag Release Candidate Notes

Status: self-test fixture.
EOF
}

write_release_notes
write_report "$head_commit" "PASS"
"$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null

rm "$release_notes"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: missing release notes did not block release" >&2
    exit 1
fi
write_release_notes

cat >"$release_notes" <<'EOF'
# Aesynx v0.0.0 Release Candidate Notes

Status: wrong title.
EOF
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: mismatched release notes did not block release" >&2
    exit 1
fi
write_release_notes

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

cat >"$report" <<EOF
Tag: $tag
Commit: $head_commit
Status: PASS
Date: 2026-06-12
Scope: self-test fixture
EOF
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: missing tester did not block release" >&2
    exit 1
fi

{
    printf 'Tag: %s\n' "$tag"
    printf 'Commit: %s\n' "$head_commit"
    printf 'Status: PASS\n'
    printf 'Tester:   \n'
    printf 'Date: 2026-06-12\n'
    printf 'Scope: self-test fixture\n'
} >"$report"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: blank tester did not block release" >&2
    exit 1
fi

cat >"$report" <<EOF
Tag: $tag
Commit: $head_commit
Status: PASS
Tester: release-readiness self-test
Date: 12-06-2026
Scope: self-test fixture
EOF
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: malformed date did not block release" >&2
    exit 1
fi

cat >"$report" <<EOF
Tag: $tag
Commit: $head_commit
Status: PASS
Tester: release-readiness self-test
Date: 2026-06-12
EOF
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: missing scope did not block release" >&2
    exit 1
fi

{
    printf 'Tag: %s\n' "$tag"
    printf 'Commit: %s\n' "$head_commit"
    printf 'Status: PASS\n'
    printf 'Tester: release-readiness self-test\n'
    printf 'Date: 2026-06-12\n'
    printf 'Scope:   \n'
} >"$report"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: blank scope did not block release" >&2
    exit 1
fi

write_report "$head_commit" "PASS"
git tag "$tag"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: existing local tag did not block release" >&2
    exit 1
fi
git tag -d "$tag" >/dev/null

cat >Cargo.toml <<'EOF'
[workspace]
members = ["crates/example"]
EOF
write_report "$head_commit" "PASS"
if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: missing SBOM did not block release" >&2
    exit 1
fi
mkdir -p sbom
printf '{"SPDXID":"SPDXRef-DOCUMENT"}\n' >sbom/aesynx.spdx.json
"$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null

write_report "$head_commit" "PASS"
cat >PENTEST.md <<'EOF'
temporary findings must block release readiness
EOF

if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: root PENTEST.md did not block release" >&2
    exit 1
fi

echo "release readiness tests: ok"
