use core::arch::asm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Port {
    address: u16,
}

impl Port {
    pub(crate) const fn new(address: u16) -> Self {
        Self { address }
    }

    pub(crate) fn read_u8(self) -> u8 {
        let value: u8;
        // SAFETY: This is the admitted x86_64 port-I/O boundary. Callers use
        // fixed legacy UART port addresses during early single-core boot, and
        // the instruction does not touch Rust memory.
        unsafe {
            asm!(
                "in al, dx",
                in("dx") self.address,
                out("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    pub(crate) fn write_u8(self, value: u8) {
        // SAFETY: This is the admitted x86_64 port-I/O boundary. Callers use
        // fixed legacy UART port addresses during early single-core boot, and
        // the instruction does not touch Rust memory.
        unsafe {
            asm!(
                "out dx, al",
                in("dx") self.address,
                in("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
