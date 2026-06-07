# Aesynx Licensing Notes

Status: maintainer decision support

The proposed project license is `EUPL-1.2`.

This is not legal advice. For a final licensing position, especially around
kernel modules, drivers, firmware, SaaS/network use, or vendor driver packages,
ask a qualified lawyer.

## Why EUPL-1.2 Fits The Project Direction

EUPL-1.2 is a reciprocal open-source license created by the European
Commission. It is a reasonable fit for a security-sensitive European OS project
that wants source-code sharing for modifications while keeping compatibility
with several other reciprocal licenses.

## Driver Boundary

Aesynx is explicitly designed so drivers can be separate driver packages or
services that talk to the kernel through a stable capability/device ABI.

That architecture helps create a clean boundary:

- The kernel can be EUPL-1.2.
- Independently written drivers can be distributed separately.
- Driver services can receive MMIO, IRQ, DMA, object, and service capabilities
  without modifying kernel source.
- Vendor driver packages can use a stable service ABI instead of linking into
  unrestricted kernel internals.

Whether a specific driver is a derivative work of the kernel is a legal
question, not only a technical one. The cleaner and more stable the ABI/service
boundary is, the stronger the engineering argument that a driver can be
independent, but the license analysis still depends on distribution, linking,
headers/ABI definitions, generated bindings, and jurisdiction.

## Practical Recommendation

Use EUPL-1.2 for the Aesynx kernel and core source if the maintainer wants a
European reciprocal license.

Keep driver-facing ABI files intentionally small, stable, documented, and
separable. Add an explicit driver-package licensing policy later before inviting
third-party vendors or closed driver services.

