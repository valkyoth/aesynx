#!/usr/bin/env sh
set -eu

required_files='
README.md
SECURITY.md
LICENSE
deny.toml
docs/IMPLEMENTATION_PLAN.md
docs/RELEASE_PLAN.md
docs/ARCHITECTURE_DECISIONS.md
docs/userspace-vision.md
docs/threat-model.md
docs/unsafe-policy.md
docs/supply-chain-security.md
docs/security-controls.md
docs/modularity-policy.md
docs/kernel-engineering-policy.md
security/pentest/README.md
'

for file in $required_files; do
    if [ ! -s "$file" ]; then
        echo "security policy: required file missing or empty: $file" >&2
        exit 1
    fi
done

retired_name_pattern='Nex''us|nex''us|Syn''apse|syn''apse|syn''sh|unknown-nex''us'
if grep -RInE "$retired_name_pattern" README.md docs SECURITY.md Cargo.toml deny.toml 2>/dev/null; then
    echo "security policy: retired project naming found" >&2
    exit 1
fi

if grep -RInE 'PRIVATE KEY|BEGIN RSA|BEGIN EC PRIVATE|BEGIN OPENSSH PRIVATE|api[_-]?key|secret[[:space:]]*=|token[[:space:]]*=' . \
    --exclude-dir=.git \
    --exclude-dir=.cargo-deny-advisory-dbs \
    --exclude='validate-security-policy.sh' \
    --exclude='*.md' \
    --exclude='*.toml' \
    --exclude='*.yml' \
    --exclude='*.yaml' 2>/dev/null; then
    echo "security policy: possible secret material found" >&2
    exit 1
fi

if ! grep -q 'EUPL-1.2' deny.toml; then
    echo "security policy: deny.toml must include EUPL-1.2 in license policy" >&2
    exit 1
fi

if ! grep -q 'unsafe' docs/unsafe-policy.md; then
    echo "security policy: unsafe policy must document unsafe-code handling" >&2
    exit 1
fi

echo "security policy: ok"
