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
cat >"security/pentest/$tag.md" <<EOF
Tag: $tag
Commit: $head_commit
Status: PASS
Tester: release-readiness self-test
Date: 2026-06-12
Scope: self-test fixture
EOF

"$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null

cat >PENTEST.md <<'EOF'
temporary findings must block release readiness
EOF

if "$root/scripts/validate-release-readiness.sh" "$tag" >/dev/null 2>&1; then
    echo "release readiness tests: root PENTEST.md did not block release" >&2
    exit 1
fi

echo "release readiness tests: ok"
