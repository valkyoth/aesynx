# Aesynx SDK Roadmap

Status: future design direction

Aesynx needs an SDK so application developers target stable OS interfaces
instead of kernel-private modules, copied C headers, raw syscall numbers, or
ad hoc package metadata.

The SDK should make native and WASM app development feel normal while preserving
the Aesynx security model: no ambient filesystem, network, device, or IPC
authority by default.

## Goals

- Give Rust developers stable crates for Aesynx ABI and runtime access.
- Give WASM developers a portable component path with Aesynx host interfaces.
- Define target triples, startup rules, app manifests, package manifests, and
  capability manifests as first-class project artifacts.
- Make a minimal app buildable, packageable, inspectable, and runnable in QEMU.
- Keep all developer-facing APIs capability-native and object-native from day
  one.

## Core SDK Pieces

| Piece | Purpose |
| --- | --- |
| `aesynx-abi` | Stable handles, object IDs, capability IDs, syscall/message numbers, value-schema IDs, wire formats, and error codes. |
| `aesynx-rt` | Safe Rust runtime wrapper for startup info, capabilities, structured channels, logging, panic reporting, allocation hooks, and raw syscall/IPC entry points. |
| Rust targets | Native app targets such as `x86_64-unknown-aesynx` and later `aarch64-unknown-aesynx`. |
| WASM profile | Portable component profile such as `wasm32-wasip2-aesynx`, mapped onto Aesynx host calls. |
| Startup/linking | Userspace entry and linker rules owned by `aesynx-rt`, similar in purpose to `crt0` but not C-header-driven. |
| App templates | Native command, native service, WASM component, and driver-service templates. |
| Manifest tooling | Validation for artifact target, entry point, exports, requested capabilities, SBOM, provenance, and signatures. |

## Native Rust Apps

Native Rust apps should depend on `aesynx-abi` and `aesynx-rt`.

The intended long-term flow:

```bash
cargo aesynx new hello --kind native-command
cargo build --target x86_64-unknown-aesynx
aepkg build
aepkg inspect target/aesynx/hello.aepkg
aepkg run target/aesynx/hello.aepkg
```

The first implementation can use repo-owned JSON targets and `build-std`
experiments. Upstream Rust target support is a later ecosystem milestone.

Native does not mean unrestricted. The launcher gives a native app an explicit
startup capability bundle. Missing authority becomes a structured
`CapabilityDenied` error.

## WASM Components

WASM should be the default untrusted extension and automation path.

The intended long-term flow:

```bash
cargo aesynx new log-view --kind wasm-component
cargo build --target wasm32-wasip2-aesynx
aepkg build
```

The Aesynx WASM host profile should expose object, capability, structured value,
telemetry, time, and service-queue calls. It should not silently inherit
POSIX-shaped filesystem or socket assumptions from generic WASI.

## App Manifest

A minimal app manifest should include:

```toml
[app]
name = "hello"
version = "0.1.0"
kind = "native-command"
target = "x86_64-unknown-aesynx"
entry = "aesynx:main"

[exports]
commands = ["hello"]
schemas = []

[capabilities]
storage_read = []
storage_write = []
network = []
ipc = []
device = []

[supply_chain]
sbom = "spdx:sha256:..."
provenance = "slsa:sha256:..."
```

Rules:

- Capability declarations are requests, not grants.
- Manifest validation rejects undeclared ambient authority.
- Package names are not security identity; signed hashes and policy are.
- Native and WASM apps use the same manifest model where possible.
- Driver-service manifests extend this model with hardware IDs and MMIO/IRQ/DMA
  requests.

## Developer API Shape

A native Rust hello-world should eventually look like:

```rust
use aesynx_rt::{env, Result};

fn main() -> Result<()> {
    let stdout = env::stdout()?;
    stdout.write_str("hello from Aesynx\n")
}
```

A denied capability should be explicit:

```rust
use aesynx_rt::{cap, Result};

fn main() -> Result<()> {
    let logs = cap::request("telemetry:boot-log")?;
    logs.read_stream()?;
    Ok(())
}
```

If the launcher did not grant `telemetry:boot-log`, the request fails with a
structured error that `aesh`, GUI tools, and AI explanations can render without
parsing arbitrary text.

## Release-Plan Hook

The first concrete SDK milestone is `v0.47.1 - Aesynx SDK And App Template`.
That milestone should not claim a complete app ecosystem. It only needs to
prove that an external developer has a documented, reproducible path for:

- Creating a minimal app.
- Building against the planned target/runtime.
- Producing a manifest.
- Validating that the manifest requests no ambient authority.
- Running or simulating the app path in QEMU or a host-side placeholder.

## Non-Goals

- POSIX headers as the native development model.
- Linux binary compatibility as the first app path.
- Ambient `/usr/bin`, `/lib`, `$PATH`, sockets, or home-directory access.
- Dynamic native shared libraries in the first SDK.
- Upstream Rust target support before the repo-owned target path works.
