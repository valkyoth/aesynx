# Aesynx Hosted Execution Roadmap

Status: design direction

Aesynx should eventually support container-like and emulation-like workflows,
but the native model should not copy Linux containers directly. Linux
containers are built around Linux kernel primitives such as namespaces, cgroups,
mounts, and process capabilities. Aesynx has a different authority model:
object roots, explicit capabilities, service queues, typed IPC, and immutable
storage.

The modern Aesynx direction should be **capsules**: isolated execution
environments with explicit object roots, capability manifests, resource budgets,
and virtualized service endpoints.

## Native Capsules

A native Aesynx capsule is the closest equivalent to a container:

- It runs on the Aesynx kernel, not inside a guest kernel.
- It has its own principal/root capability set.
- It sees a scoped object namespace, not a global filesystem.
- It receives explicit service endpoints for network, storage, clock, entropy,
  telemetry, and console.
- It has CPU, memory, IPC, object-store, and device-resource budgets.
- It can run native Aesynx components and WASM components.
- It emits provenance and audit events for all granted authority.

This is the preferred long-term replacement for "run this container" in native
Aesynx.

## Hosted Aesynx Runtime

For development and portability, Aesynx should also support a hosted runtime
that runs Aesynx userspace concepts on another host kernel.

Example:

```text
aesynx-host run --root object:... /bin/aesh
```

That runtime could run on Linux, macOS, Windows, or CI machines. It would map
Aesynx object storage, value/schema ABI, component manifests, and capability
checks onto host facilities. This is similar in spirit to running a small OS
environment under a host kernel, but the compatibility boundary is the Aesynx
component/object ABI rather than POSIX.

The hosted runtime is useful for:

- Developing shell commands before the kernel is bootable.
- Testing object-store logic on ordinary CI.
- Running Aesynx automation on existing machines.
- Debugging components with host tooling.
- Reusing the same component package in hosted and native Aesynx modes.

## Micro-VM And Foreign Compatibility

Some workloads need a foreign kernel or stronger isolation than a capsule. That
should be a later layer:

- Micro-VM service for running small guest kernels.
- Virtio-style virtual devices backed by Aesynx services.
- Capability-scoped storage and network backends.
- Optional Linux/POSIX compatibility service after the native model is mature.

This is not a 1.0 goal. It should not shape the early kernel ABI.

## Required Kernel Primitives

Capsules require kernel support for:

- Principals and capability roots.
- Isolated address spaces.
- Object namespace roots.
- Resource budgets and accounting.
- Service endpoint virtualization.
- Revocation and suspend/kill lifecycle.
- Structured telemetry and audit events.
- Deterministic startup manifests.

These primitives are useful for native security even before compatibility or
hosted workflows exist.

## Non-Goals

- Running OCI/Linux containers unchanged in 1.0.
- Treating POSIX paths as the native capsule namespace.
- Giving capsules ambient filesystem or network authority.
- Making Linux syscall compatibility part of the core kernel ABI.
