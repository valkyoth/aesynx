# Aesynx Threat Model

Status: baseline

This document defines the threats Aesynx is expected to handle as the project
grows. It is not a claim that the implementation handles all of them today.

## Assets

Aesynx protects:

- Kernel memory and control flow.
- Capability objects and revocation state.
- Address spaces, page tables, and object mappings.
- IPC queues and message integrity.
- Driver MMIO, IRQ, and DMA authority.
- Object graph metadata and content.
- WASM component manifests and granted authorities.
- Telemetry streams and AI policy inputs.
- Build, release, and supply-chain metadata.

## Trust Boundaries

Primary trust boundaries:

- Bootloader to kernel handoff.
- Kernel to userspace runtime.
- Kernel to driver services.
- Driver service to hardware device.
- Userspace command to shell runtime.
- WASM component to host capability surface.
- Telemetry collection to AI-assisted policy engine.
- Repository source to release artifact.

## Attacker Model

Aesynx assumes attackers may:

- Run untrusted userspace components.
- Submit malicious WASM components or command manifests.
- Attempt to forge, reuse, or over-broaden capabilities.
- Send malformed IPC messages.
- Trigger parser edge cases in manifests and structured pipeline data.
- Abuse driver interfaces, MMIO windows, IRQ routing, or DMA authority.
- Try to poison telemetry used by AI-assisted decisions.
- Modify dependency or build metadata through supply-chain compromise.
- Exploit kernel bugs through QEMU inputs before hardware support exists.

Aesynx does not assume early versions can resist:

- Physical attacks.
- Malicious firmware.
- Compromised host operating systems running QEMU.
- Side-channel attacks not explicitly modeled in a given release.
- Arbitrary hardware DMA without IOMMU enforcement or trusted-degraded mode.

## Security Objectives

The design should enforce:

- No ambient authority by default.
- Capability checks before object, memory, device, and IPC access.
- Generation and revocation checks for authority-bearing handles.
- Explicit authority transfer through audited grant paths.
- Deterministic fallback when AI-assisted policy is unavailable or rejected.
- Structured telemetry redaction before data reaches assistive tools.
- Small modules that can be reviewed, fuzzed, and model-tested.
- Release tags blocked until checks and pentest evidence are complete.

## Non-Goals Before 1.0

Before the QEMU 1.0 release, Aesynx does not promise:

- POSIX or Unix compatibility.
- General desktop usability.
- Production hardware certification.
- Full formal verification.
- Complete side-channel resistance.
- Broad third-party driver support.

## Required Evidence

Security-sensitive milestones should add:

- Unit or model tests for authority checks.
- Negative tests for malformed inputs.
- Fuzz targets when parsers appear.
- QEMU smoke tests when boot behavior exists.
- A release pentest report before tagging.
