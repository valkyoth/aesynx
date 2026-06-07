# Pull Request

## Summary

Describe what changed and why.

## Type

- [ ] Boot/kernel
- [ ] Memory/capability/IPC
- [ ] Userspace/WASM/AI
- [ ] Driver/device
- [ ] Documentation
- [ ] Refactor
- [ ] Dependency update
- [ ] Security hardening

## Checklist

- [ ] I kept the change scoped to Aesynx's architecture and planning docs.
- [ ] I split code into focused crates/modules and avoided giant `.rs` files.
- [ ] I updated docs or release-plan milestones when behavior changed.
- [ ] I added or updated tests/model checks for behavior changes.
- [ ] I ran `scripts/checks.sh`.
- [ ] I checked dependency/license impact when adding or updating crates.
- [ ] I documented unsafe code in `docs/unsafe-policy.md`.
- [ ] I did not commit secrets, private keys, tokens, local runtime data, QEMU
      images, crash dumps, or generated release artifacts.

## Security Notes

Describe any security-sensitive impact. Mention boot, memory, page tables,
capabilities, IPC, drivers, DMA, userspace ABI, WASM, AI policy, telemetry,
build tooling, or dependencies if they are touched.

## Follow-Up

List known remaining work or intentionally deferred tasks.
