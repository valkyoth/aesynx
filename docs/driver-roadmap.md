# Aesynx Driver Roadmap

Status: future design direction

Aesynx should treat drivers as capability-scoped components, not as informal
kernel plugins. The project can build a small set of first-party bootstrap
drivers for QEMU and core hardware, but broad hardware support must eventually
come from a community and vendor driver ecosystem.

## Core Position

The kernel should own isolation and authority. Drivers should own device
protocol logic.

Kernel responsibilities:

- Device discovery authority.
- Capability grants for MMIO, port I/O, IRQs, DMA, firmware, and bus access.
- IOMMU or bounce-buffer enforcement for DMA-capable devices.
- Driver service isolation, scheduling, restart, and revocation.
- Driver package verification and policy admission.
- Audit events for probe, bind, grant, crash, restart, update, and revoke.

Driver responsibilities:

- Hardware-specific protocol handling.
- Device initialization and reset logic.
- Queue and buffer management within granted authority.
- Exposing stable class-service APIs such as network, block, input, or graphics.
- Declaring required capabilities and supported hardware IDs.

The kernel must not become a monolithic driver warehouse. A boot image may
package selected drivers together, but each driver remains independently
identified, signed, updateable, removable, and rollback-capable.

## Repository Shape

Core OS and driver code should be separated from day one:

```text
crates/
|-- aesynx-kernel/
|-- aesynx-device/
|-- aesynx-driver-api/
`-- core OS model crates

drivers/
|-- README.md
|-- common/
|   |-- aesynx-driver-api/
|   `-- aesynx-driver-test/
|-- bus/
|   |-- pci/
|   |-- usb/
|   |-- xhci/
|   `-- virtio/
|-- console/
|   |-- virtio-serial/
|   `-- uart16550/
|-- network/
|   |-- virtio-net/
|   |-- e1000/
|   `-- rtl8139/
|-- storage/
|   |-- virtio-blk/
|   |-- usb-mass-storage/
|   |-- nvme/
|   `-- ahci/
|-- gpu/
|   |-- framebuffer/
|   |-- virtio-gpu/
|   |-- amd/
|   |-- intel/
|   `-- nvidia/
|-- input/
|   |-- ps2/
|   `-- usb-hid/
`-- firmware/
    |-- acpi/
    `-- uefi/
```

Early releases can keep tiny bootstrap shims in architecture or kernel crates
when that is necessary for first boot. Once a driver grows beyond boot
diagnostics or QEMU smoke, move it toward `drivers/` and the driver-service
model.

The in-tree `drivers/` area is an early ABI-shaping convenience, not the final
ecosystem boundary. QEMU and virtio drivers should stay in this repository while
the driver API, capability grants, QEMU CI, package manifests, and service model
are still changing. They should be split into separate repositories once the
public driver ABI is stable enough for external packages.

Split-out triggers:

- `aesynx-driver-api` exists and is versioned.
- Driver manifests are enforced.
- MMIO, IRQ, DMA, firmware, and bus capability grants are stable.
- Driver packages can be signed, installed, updated, removed, and rolled back.
- QEMU driver CI can run against the public driver ABI without kernel-private
  hooks.
- Kernel releases no longer require driver source edits for ordinary driver
  updates.

Likely future organization layout:

```text
aesynx/kernel          # or aesynx/multikernel
aesynx/drivers-qemu
aesynx/drivers-virtio
aesynx/driver-sdk
aesynx/drivers-community
```

The current repository may eventually be renamed under an `aesynx/`
organization, most likely to `aesynx/kernel` or `aesynx/multikernel`, once the
broader OS ecosystem has separate repositories.

## QEMU First Driver Set

The first driver target is QEMU, so the early device set should prefer virtio
and simple virtual hardware over legacy PC devices.

Planned early order:

1. Bootstrap serial classification: current COM1 logging remains an early
   diagnostic path, not the long-term serial service.
2. Bootloader framebuffer wrapper: enough for early display metadata and simple
   output; not a GPU stack.
3. PCI or virtio-mmio discovery, depending on the QEMU machine profile chosen
   for the release.
4. MMIO and IRQ capabilities.
5. Virtio common transport and queue support.
6. `virtio-rng` for entropy service work.
7. `virtio-blk` for the first QEMU persistence path.
8. `virtio-serial` for a non-legacy virtual serial service channel separate
   from bootstrap COM1 logs.
9. `virtio-net` for the first QEMU networking path.
10. `virtio-gpu` for a basic QEMU display resource and scanout path.
11. Optional PS/2 keyboard/mouse for local QEMU input before the USB stack.
12. xHCI discovery, then USB HID, USB mass storage, and USB serial classes.

Explicit non-goals for the first QEMU driver wave:

- ICH9/Intel HDA audio.
- Full GPU acceleration, 3D, shader execution, or compositor protocols.
- Vendor GPU stacks.
- Old UHCI/OHCI/EHCI-first USB support.
- USB mass storage as the first storage path.

Early keyboard input should not wait for USB. The first interactive paths are
serial/virtio-serial for CI and automation, then optionally PS/2 keyboard/mouse
for local QEMU use. USB is still planned, but later. The preferred USB path is
xHCI first, then class drivers such as USB HID, USB mass storage, and USB serial
adapters. Reading a USB stick therefore requires the xHCI controller driver,
USB enumeration, endpoint/transfer management, the mass-storage class, and a
block-service binding. For QEMU and `v1.0.0`, `virtio-blk` is the simpler first
block device.

## Driver Classes

| Class | Examples | Default placement | Policy |
| --- | --- | --- | --- |
| Bootstrap-critical | serial, early framebuffer, boot block read | kernel or trusted boot service | Minimal, audited, no broad ecosystem |
| First-party trusted | virtio, PCI bus, NVMe basics | `drivers/` package/service | Signed by Aesynx, tested in CI |
| Community | open NIC, input, storage, GPU helpers | external package | Signed publisher, restricted caps, reviewable source preferred |
| Vendor | proprietary GPU or Wi-Fi driver | external package | Vendor key, explicit user/admin trust, tainted-state telemetry |

Closed-source vendor drivers may be supported only as isolated signed driver
services. They must not be linked into the kernel and must not receive ambient
kernel authority.

## Driver Packages

External driver installation should feel simple for users:

```text
aepkg search driver realtek
aepkg install driver:rtl8125
aepkg remove driver:rtl8125
aepkg update --track vendor
aesh drivers
aesh driver bind pci:10ec:8125 --driver driver:rtl8125
```

Under the hood this is not a Linux-style kernel module install. Installing a
driver publishes a new declarative generation:

1. Fetch signed driver manifest from an enabled track.
2. Verify package hash, publisher signature, SBOM, provenance, and registry
   inclusion.
3. Match hardware IDs against discovered devices.
4. Compare requested capabilities with local trust policy.
5. Ask for user/admin approval if the driver is community, vendor, proprietary,
   or asks for sensitive resources.
6. Stage the driver as an immutable object.
7. Publish a new system generation that makes the driver selectable.
8. Driver manager starts it as an isolated service and grants only exact device
   capabilities.

Removing a driver publishes a new generation without that driver, drains active
devices, revokes IRQ/MMIO/DMA caps, fences interrupts and DMA, and rolls back
if quiesce fails.

## Driver Manifest

A future driver manifest should include at least:

```toml
[package]
name = "rtl8125"
kind = "driver-service"
track = "community"
version = "1.2.0"
publisher = "did:aesynx:pub:example"

[driver]
class = "network"
abi = "aesynx-driver-net.v1"
entry = "service:rtl8125"
restart = "allowed"
hot_unplug = "supported"

[[hardware]]
bus = "pci"
vendor = "0x10ec"
device = "0x8125"

[capabilities]
pci_config = ["device-only"]
mmio = ["bar0"]
irq = ["device"]
dma = { required = true, isolation = "iommu-or-bounce-buffer" }
network_class = true
firmware = []

[supply_chain]
sbom = "spdx:sha256:..."
source = "https://example.invalid/rtl8125"
provenance = "slsa:sha256:..."
transparency_entry = "rekor-like:..."

[[signatures]]
algorithm = "policy-selected"
key_id = "did:aesynx:pub:example#key-1"
value = "base64:..."
```

Capability declarations are requests, not grants. The driver manager and local
policy decide what the driver actually receives.

## Discovery And Binding

The driver manager should separate discovery from binding:

1. Bus service discovers devices.
2. Device objects are created with stable identities and resource descriptors.
3. Package service searches local and enabled remote tracks for matching driver
   manifests.
4. Policy ranks matches: core, official, sovereign, community, vendor.
5. Installer or admin UI presents recommended drivers and risk labels.
6. Binding grants capabilities and starts the driver service.
7. Class service exposes a stable interface to userspace.

This supports an installer driver-selection screen without making the kernel
compile in every possible driver:

```text
Detected device                 Recommended driver        Track
Realtek RTL8125 NIC             rtl8125                   community
Virtio block controller         virtio-blk                core
AMD GPU                         amd-gpu                   vendor/official
```

## Security Rules

- No untrusted driver may run in kernel mode by default.
- No third-party driver may require editing kernel source.
- No driver receives raw physical memory.
- No driver receives all devices, all IRQs, or unrestricted PCI config access.
- DMA-capable external drivers require IOMMU enforcement or a documented
  bounce-buffer fallback.
- Driver packages must be signed and track-scoped.
- Driver package names are not security identity; hashes, signatures, hardware
  IDs, and policy are.
- Vendor/proprietary drivers must produce a visible trust/taint state.
- Firmware blobs are separate package objects with their own hashes and
  licenses.
- Driver crashes must be contained to the driver service where possible.
- Revocation must drain queues, stop DMA, disable IRQs, and revoke MMIO/DMA
  windows before unload.
- Production DMA revocation also disables PCI bus mastering, performs
  device-specific or function-level reset where available, unmaps IOMMU entries
  and waits for completed IOTLB invalidation, invalidates or prohibits ATS,
  PASID, and PRI, fences interrupt-remapping/MSI/MSI-X delivery, and assigns
  new device plus interrupt incarnations before restart. Devices that cannot be
  reliably quiesced, reset, or fenced fail closed instead of being rebound.

## Development Model

Aesynx cannot and should not write every driver itself. First-party effort
should focus on:

- QEMU and virtio drivers needed for `v1.0.0`.
- Bus and class APIs.
- The driver package ABI.
- Test harnesses and fake devices.
- Security policy, IOMMU/DMA enforcement, and revocation.
- Documentation that lets community and vendor authors write drivers without
  changing the kernel.

External contributors should be able to build a driver with:

```text
cargo new-driver --class network --bus pci
cargo test -p aesynx-driver-rtl8125
aepkg build-driver
aepkg publish --track community
```

Those commands are placeholders, but the experience is the target: driver
authors build packages against a stable driver ABI, users install them through
the package manager, and the kernel remains small.

## 1.0 Boundary

The QEMU `v1.0.0` release should not require the complete external driver
ecosystem. It should have:

- Clear `drivers/` layout.
- Driver model documentation.
- Bootstrap trusted QEMU/virtio driver set.
- Device objects and lifecycle states.
- MMIO/IRQ capability model.
- DMA policy labeled trusted/degraded if IOMMU is not complete.
- Package/manifest fields that do not block future external drivers.

Broad community and vendor driver installation belongs after the base package
manager, persistent object store, and driver service isolation exist.
