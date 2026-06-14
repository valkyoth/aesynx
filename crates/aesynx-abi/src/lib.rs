#![no_std]
#![deny(unsafe_code)]

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident, $inner:ty) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[repr(transparent)]
        pub struct $name($inner);

        impl $name {
            #[must_use]
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn get(self) -> $inner {
                self.0
            }
        }
    };
}

id_type!(CoreId, u32);
id_type!(CpuHardwareId, u64);
id_type!(
    /// Physical address value.
    ///
    /// This is a raw numeric address wrapper. Callers at privilege boundaries
    /// must validate architecture-specific address-width and memory-map
    /// constraints before using it for mapping, DMA, or device access.
    PhysAddr,
    u64
);
id_type!(
    /// Virtual address value.
    ///
    /// This is a raw numeric address wrapper. On x86_64, callers at privilege
    /// boundaries such as syscalls or untrusted IPC must validate canonical
    /// form before the value is used as a pointer or mapping address.
    VirtAddr,
    u64
);
impl VirtAddr {
    /// Returns `Some` only when `value` is canonical under the x86_64 48-bit
    /// virtual-address rule.
    ///
    /// This helper is intended for privilege-boundary validation. `new` remains
    /// available for raw numeric values that have already been checked by an
    /// architecture-specific layer.
    #[must_use]
    pub const fn new_x86_64_canonical(value: u64) -> Option<Self> {
        let sign_extended = ((value as i64) << 16) >> 16;
        if sign_extended as u64 == value {
            Some(Self(value))
        } else {
            None
        }
    }
}
id_type!(PhysFrame, u64);
id_type!(Page, u64);
id_type!(ObjectId, u128);
id_type!(CapId, u64);
id_type!(PrincipalId, u64);
id_type!(MessageId, u64);
id_type!(DeviceId, u128);
id_type!(DmaAddr, u64);
id_type!(DmaDomainId, u64);
id_type!(IrqLine, u32);
id_type!(TaskId, u64);
id_type!(ProcessId, u64);
id_type!(PolicyId, u64);
id_type!(ModelId, u128);

pub const ROOT_CORE: CoreId = CoreId::new(0);

#[cfg(test)]
mod tests {
    use super::VirtAddr;

    #[test]
    fn virt_addr_x86_64_canonical_constructor_accepts_sign_extended_values() {
        assert_eq!(
            VirtAddr::new_x86_64_canonical(0x0000_7fff_ffff_f000),
            Some(VirtAddr::new(0x0000_7fff_ffff_f000))
        );
        assert_eq!(
            VirtAddr::new_x86_64_canonical(0xffff_8000_0000_0000),
            Some(VirtAddr::new(0xffff_8000_0000_0000))
        );
    }

    #[test]
    fn virt_addr_x86_64_canonical_constructor_rejects_noncanonical_values() {
        assert_eq!(VirtAddr::new_x86_64_canonical(0x0000_8000_0000_0000), None);
        assert_eq!(VirtAddr::new_x86_64_canonical(0xffff_7fff_ffff_ffff), None);
    }
}
