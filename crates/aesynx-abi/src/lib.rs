#![no_std]
#![deny(unsafe_code)]

macro_rules! id_type {
    ($name:ident, $inner:ty) => {
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
id_type!(PhysAddr, u64);
id_type!(VirtAddr, u64);
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
