use aesynx_abi::{PhysAddr, VirtAddr};

const KERNEL_VIRT: VirtAddr = VirtAddr::new(0xffff_9000_0000_0000);
const KERNEL_PHYS: PhysAddr = PhysAddr::new(0x0020_0000);

mod mapper;
mod raw;
