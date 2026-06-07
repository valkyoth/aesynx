#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CpuHardwareId, PhysAddr, VirtAddr};

#[derive(Debug, Eq, PartialEq)]
pub struct BootInfo<'a> {
    pub arch: ArchKind,
    pub platform: PlatformKind,
    pub memory_map: &'a [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub rsdp: Option<PhysAddr>,
    pub device_tree: Option<PhysAddr>,
    pub cpu_topology: &'a [CpuInfo],
    pub kernel_image: KernelImageInfo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchKind {
    X86_64,
    Aarch64,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKind {
    Qemu,
    Uefi,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub len: u64,
    pub kind: MemoryRegionKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    Kernel,
    Bootloader,
    Framebuffer,
    Acpi,
    Bad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramebufferInfo {
    pub base: PhysAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuInfo {
    pub hardware_id: CpuHardwareId,
    pub enabled: bool,
}

#[derive(Eq, PartialEq)]
pub struct KernelImageInfo {
    virt_start: VirtAddr,
    virt_end: VirtAddr,
    phys_start: PhysAddr,
}

impl KernelImageInfo {
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn new(
        virt_start: VirtAddr,
        virt_end: VirtAddr,
        phys_start: PhysAddr,
    ) -> Self {
        Self {
            virt_start,
            virt_end,
            phys_start,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn virt_start(&self) -> VirtAddr {
        self.virt_start
    }

    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn virt_end(&self) -> VirtAddr {
        self.virt_end
    }

    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn phys_start(&self) -> PhysAddr {
        self.phys_start
    }
}

impl core::fmt::Debug for KernelImageInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("KernelImageInfo(redacted)")
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use aesynx_abi::{PhysAddr, VirtAddr};

    use super::KernelImageInfo;

    #[test]
    fn kernel_image_debug_redacts_addresses() {
        let info = KernelImageInfo::new(VirtAddr::new(1), VirtAddr::new(2), PhysAddr::new(3));
        let mut output = FixedBuf::default();

        assert_eq!(fmt::write(&mut output, format_args!("{info:?}")), Ok(()));
        assert_eq!(output.as_str(), "KernelImageInfo(redacted)");
    }

    #[derive(Default)]
    struct FixedBuf {
        bytes: [u8; 32],
        len: usize,
    }

    impl FixedBuf {
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
        }
    }

    impl fmt::Write for FixedBuf {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            if self.len + value.len() > self.bytes.len() {
                return Err(fmt::Error);
            }

            let end = self.len + value.len();
            self.bytes[self.len..end].copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
