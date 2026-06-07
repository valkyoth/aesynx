# Aesynx Kernel Engineering Policy

Status: hard rule

Aesynx kernel-side code must stay small, explicit, reviewable, and independent
from ordinary operating-system assumptions. Convenience is not enough reason to
add dependencies, use `std`, or introduce unsafe code.

## Hard Rules

Crates under `crates/` are kernel, runtime, ABI, or model-adjacent crates. They
must:

- Use `#![no_std]`.
- Use `#![deny(unsafe_code)]` unless an unsafe exception is documented first.
- Avoid `std` imports entirely.
- Prefer Aesynx-owned primitives for kernel-critical behavior.
- Keep external dependencies out unless a serious reason is documented.

Host-only tooling under `tools/`, release scripts, CI helpers, fuzz harnesses,
and future host models may use `std`, but external dependencies still require a
reviewed reason when they affect build, release, parsing, fuzzing, or security
evidence.

## Build Our Own

Aesynx should own these primitives:

- Capability model and revocation.
- Kernel object identities and object graph.
- Scheduler policy and task state.
- IPC messages, queues, and service protocols.
- Memory-management models.
- Driver authority model.
- Telemetry schemas.
- AI policy boundary and deterministic fallback.
- Native shell/runtime ABI.

Owning these areas keeps the security model coherent. A third-party crate that
silently imports another authority model, allocation model, parser behavior, or
runtime assumption is a design risk.

## External Dependency Exceptions

External crates are allowed only when the reason is stronger than convenience.
Acceptable reasons include:

- A mature, audited implementation is materially safer than a new local one.
- The crate is host-only release, test, fuzzing, or verification tooling.
- The crate implements a standard where mistakes are likely and expensive.
- The dependency is temporary and has a removal plan.

Every external crate must have an exception entry before it is added.

Exception format:

```text
Crate:
Used by:
Scope:
Reason:
Why not local:
Security review:
License:
Review deadline:
Removal condition:
```

Current external dependency exceptions:

- None.

## Unsafe Exceptions

Unsafe code is expected eventually for CPU entry, interrupt handling, page
tables, MMIO, context switching, allocators, and carefully reviewed queue
internals. It is not allowed until the exact boundary is documented in
`docs/unsafe-policy.md`.

## Validator

`scripts/validate-kernel-policy.sh` enforces the current baseline:

- Every crate under `crates/` has `#![no_std]`.
- Every crate under `crates/` has `#![deny(unsafe_code)]` or
  `#![forbid(unsafe_code)]`.
- No Rust file under `crates/` imports `std`.
- Non-Aesynx dependencies must have an exception entry in this document.

If a future milestone needs an exception, update the policy first, then update
the validator narrowly enough that the exception remains reviewable.
