# Aesynx Post-Quantum Readiness Roadmap

Status: future design direction

Aesynx should be post-quantum ready by design. This is mainly a cryptographic
architecture requirement, not a requirement to run directly on quantum
processors.

Quantum processors should be treated as future accelerators: if hardware
appears, Aesynx can support it with driver services, device capabilities,
scheduling policy, and a userspace runtime. The security risk that matters much
earlier is that large quantum computers would break common public-key systems
such as RSA and elliptic-curve signatures/key exchange.

## Design Goal

No Aesynx trust boundary should assume one permanent public-key algorithm,
signature size, key size, certificate format, or handshake shape.

This applies to:

- Boot capsule signatures.
- Package and registry signatures.
- Transparency-log inclusion proofs.
- Update metadata.
- Driver package manifests.
- Capability delegation tokens, if they become signed artifacts.
- Secure IPC, remote attestation, and network identity.
- Paid package entitlement receipts.
- Model/policy object signatures.

## Baseline Standards To Track

The current post-quantum baseline to track is the NIST PQC family:

- FIPS 203 / ML-KEM for key establishment.
- FIPS 204 / ML-DSA for signatures.
- FIPS 205 / SLH-DSA as a stateless hash-based signature option.

These names should be treated as policy inputs, not hardwired forever. The
project must remain able to add, remove, deprecate, and migrate algorithms by
policy and release generation.

References:

- https://www.nist.gov/news-events/news/2024/08/announcing-approval-three-federal-information-standards-fips
- https://csrc.nist.gov/pubs/fips/203/final
- https://csrc.nist.gov/pubs/fips/204/final
- https://csrc.nist.gov/pubs/fips/205/final

## Non-Negotiable Rules

- Use algorithm identifiers everywhere a key, signature, ciphertext, or KEM
  output is stored.
- Version all cryptographic object formats.
- Do not encode public-key, signature, or ciphertext sizes as tiny fixed
  buffers in stable ABIs.
- Prefer length-delimited byte slices or bounded variable-size objects with
  explicit maximums.
- Keep crypto verification behind small internal interfaces so providers can be
  replaced or upgraded.
- Separate hash identity from signature policy. Content addressing can stay
  hash-based while signature algorithms migrate.
- Support multi-signature or hybrid signature envelopes before relying on
  signed boot capsules, package registries, or update metadata.
- Reject unknown algorithms by default unless local policy explicitly admits
  them.
- Record algorithm, provider, parameter set, creation time, expiry, and
  deprecation state in trust metadata.
- Treat cryptographic migration as a generation transition with rollback and
  audit records.

## Hybrid Transition

During the migration period, Aesynx should prefer hybrid validation for
security-critical artifacts:

- Classical signature plus post-quantum signature for boot capsules.
- Classical registry signature plus post-quantum registry signature for package
  indexes.
- Classical key exchange plus ML-KEM-style key establishment for remote secure
  channels, once networking exists.

Hybrid validation prevents a premature dependency on one young algorithm family
while avoiding permanent dependence on quantum-vulnerable classical algorithms.

## Boot And Firmware Boundary

The future Rust bootloader must be crypto-agile from its first design:

- Boot capsule manifests carry a signature envelope, not a single signature
  field.
- The bootloader verifies against local boot policy that can require hybrid
  signatures.
- TPM measurement includes the selected algorithm IDs and verified manifest
  hashes.
- Secure Boot compatibility is a firmware integration detail. It must not be
  the only trust path.
- Signature verification code in the bootloader must stay tiny, reviewed, and
  replaceable.

Early Limine-based releases do not implement this yet. They must avoid
documentation or ABI choices that would block it.

## Package Manager Boundary

The package manager must assume algorithm migration from day one:

- Package manifests use signature envelopes.
- Registry snapshots use signature envelopes.
- Track policy says which algorithms are accepted per track.
- Core packages can require stricter algorithms than community packages.
- Paid entitlement receipts are signed objects with algorithm identifiers and
  expiry policy.
- SBOM/provenance objects are independently signed or referenced by immutable
  hashes.
- Self-healing must re-verify signatures against current policy before
  repairing objects from mirrors.

Package names remain advisory. Hashes, signatures, transparency policy, and
local trust roots remain authoritative.

## Kernel And Runtime Boundary

The kernel should not embed large cryptographic stacks early. Instead:

- Model cryptographic identities and policies in host-testable crates first.
- Keep boot verification, package verification, and runtime secure-channel
  verification as separate services or narrow modules.
- Put heavyweight cryptography in audited service components where possible.
- Use kernel enforcement for capabilities, isolation, revocation, and object
  identity.
- Avoid giving crypto services ambient authority over storage, network, or
  package state.

If the kernel eventually verifies signatures directly, that code becomes part
of the trusted computing base and needs dedicated unsafe, dependency, side
channel, and fuzzing review.

## Quantum Hardware Support

Quantum compute hardware is not a 1.0 kernel goal. If it becomes practical,
Aesynx should treat it as a specialized accelerator:

- Driver service with explicit device capabilities.
- Queue-based command submission.
- Capability-scoped access to experiment state and result objects.
- Scheduler awareness for scarce accelerator resources.
- No ambient access to package, key, or model stores.

Rust compiler support may matter for developer tooling, but the OS design work
is mostly driver isolation, accelerator scheduling, and userspace APIs.

## Implementation Order

1. Add crypto-agile data models for algorithm IDs, signature envelopes, and
   trust policy in host-testable crates.
2. Add tests that prove unknown algorithms are rejected and multiple signatures
   can be represented without fixed-size assumptions.
3. Add package-manifest signature-envelope fields before package installation
   exists.
4. Add boot-capsule signature-envelope fields before the future bootloader
   exists.
5. Add policy support for hybrid classical plus post-quantum requirements.
6. Add audited cryptographic provider integration only when a real verifier is
   needed.
7. Add migration tooling that can re-sign manifests and publish new generations
   without mutating existing objects.

## Release Planning

Post-quantum readiness should become an explicit review item before:

- Signed boot capsules.
- Native package registries.
- Secure update metadata.
- Remote attestation.
- Network key exchange.
- Paid marketplace receipts.
- Driver package verification.

No release should claim post-quantum security until real implementations,
provider review, test vectors, side-channel analysis, and release gates exist.
