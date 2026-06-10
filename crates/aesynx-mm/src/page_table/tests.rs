use aesynx_abi::{PhysAddr, VirtAddr};

const KERNEL_VIRT: VirtAddr = VirtAddr::new(0xffff_9000_0000_0000);
const KERNEL_PHYS: PhysAddr = PhysAddr::new(0x0020_0000);

mod alias_policy;
mod audit;
mod mapper;
mod policy;
mod presence;
mod range;
mod range_address_policy;
mod range_policy;
mod range_presence;
mod range_privilege_policy;
mod raw;
mod summary;
mod walk;
