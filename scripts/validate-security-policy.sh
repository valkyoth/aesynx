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

if grep -RInE 'PRIVATE KEY|BEGIN RSA|BEGIN EC PRIVATE|BEGIN OPENSSH PRIVATE|api[_-]?key[[:space:]]*[=:]|secret[[:space:]]*[=:]|token[[:space:]]*[=:]|password[[:space:]]*[=:]|credential[[:space:]]*[=:]' . \
    --exclude-dir=.git \
    --exclude-dir=.cargo-deny-advisory-dbs \
    --exclude-dir=target \
    --exclude-dir=sbom \
    --exclude='validate-security-policy.sh' \
    2>/dev/null; then
    echo "security policy: possible secret material found" >&2
    exit 1
fi

if [ -d .github/workflows ] && grep -RInE '^[[:space:]]*uses:[[:space:]]*[^[:space:]#]+@[A-Za-z0-9._-]+([[:space:]]*#.*)?$' .github/workflows \
    | grep -vE '@[0-9a-f]{40}([[:space:]]|$)' 2>/dev/null; then
    echo "security policy: GitHub Actions must be pinned to commit SHA" >&2
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

allowed_unsafe_files="$(cat <<'EOF'
crates/aesynx-arch-aarch64/src/lib.rs
crates/aesynx-arch-x86_64/src/descriptors.rs
crates/aesynx-arch-x86_64/src/exceptions.rs
crates/aesynx-arch-x86_64/src/exceptions/tests.rs
crates/aesynx-arch-x86_64/src/lib.rs
crates/aesynx-arch-x86_64/src/port.rs
crates/aesynx-arch-x86_64/src/registers.rs
crates/aesynx-arch-x86_64/src/timer.rs
crates/aesynx-kernel/src/limine.rs
crates/aesynx-kernel/src/main.rs
EOF
)"

actual_unsafe_files="$(
    find crates -type f -name '*.rs' -exec grep -IlE 'unsafe[[:space:]]*\{|unsafe[[:space:]]+fn|unsafe[[:space:]]+extern|#\[unsafe|allow\(unsafe_code\)|global_asm!|asm!' {} + \
        | LC_ALL=C sort
)"

if [ "$actual_unsafe_files" != "$allowed_unsafe_files" ]; then
    echo "security policy: unsafe file inventory changed; update docs/unsafe-policy.md and validate-security-policy.sh together" >&2
    echo "security policy: expected unsafe files:" >&2
    printf '%s\n' "$allowed_unsafe_files" >&2
    echo "security policy: actual unsafe files:" >&2
    printf '%s\n' "$actual_unsafe_files" >&2
    exit 1
fi

for file in $allowed_unsafe_files; do
    if ! grep -Fq "Location: $file" docs/unsafe-policy.md; then
        echo "security policy: unsafe file missing from docs/unsafe-policy.md: $file" >&2
        exit 1
    fi
done

unsafe_block_failures="$(
    find crates -type f -name '*.rs' -exec awk '
        /unsafe[[:space:]]*\{/ {
            if (previous_1 !~ /SAFETY:/ && previous_2 !~ /SAFETY:/ && previous_3 !~ /SAFETY:/ && previous_4 !~ /SAFETY:/ && previous_5 !~ /SAFETY:/ && previous_6 !~ /SAFETY:/ && previous_7 !~ /SAFETY:/ && previous_8 !~ /SAFETY:/) {
                printf "%s:%d\n", FILENAME, FNR
            }
        }
        {
            previous_8 = previous_7
            previous_7 = previous_6
            previous_6 = previous_5
            previous_5 = previous_4
            previous_4 = previous_3
            previous_3 = previous_2
            previous_2 = previous_1
            previous_1 = $0
        }
    ' {} +
)"

if [ -n "$unsafe_block_failures" ]; then
    echo "security policy: unsafe block missing nearby SAFETY comment" >&2
    printf '%s\n' "$unsafe_block_failures" >&2
    exit 1
fi

echo "security policy: ok"
