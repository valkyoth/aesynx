use core::arch::asm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdmittedPort {
    Data,
    InterruptEnable,
    FifoControl,
    LineControl,
    ModemControl,
    LineStatus,
    PicMasterCommand,
    PicMasterData,
    PicSlaveCommand,
    PicSlaveData,
}

impl AdmittedPort {
    const fn address(self) -> u16 {
        match self {
            Self::Data => 0x3f8,
            Self::InterruptEnable => 0x3f9,
            Self::FifoControl => 0x3fa,
            Self::LineControl => 0x3fb,
            Self::ModemControl => 0x3fc,
            Self::LineStatus => 0x3fd,
            Self::PicMasterCommand => 0x20,
            Self::PicMasterData => 0x21,
            Self::PicSlaveCommand => 0xa0,
            Self::PicSlaveData => 0xa1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Port {
    admitted: AdmittedPort,
}

impl Port {
    pub(crate) const fn new(admitted: AdmittedPort) -> Self {
        Self { admitted }
    }

    pub(crate) fn read_u8(self) -> u8 {
        let value: u8;
        let address = self.admitted.address();
        // SAFETY: This is the admitted x86_64 port-I/O boundary. Callers can
        // only construct ports from the fixed legacy COM1 UART and 8259 PIC
        // controller port set during early single-core boot, and the
        // instruction does not touch Rust memory.
        unsafe {
            asm!(
                "in al, dx",
                in("dx") address,
                out("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    pub(crate) fn write_u8(self, value: u8) {
        let address = self.admitted.address();
        // SAFETY: This is the admitted x86_64 port-I/O boundary. Callers can
        // only construct ports from the fixed legacy COM1 UART and 8259 PIC
        // controller port set during early single-core boot, and the
        // instruction does not touch Rust memory.
        unsafe {
            asm!(
                "out dx, al",
                in("dx") address,
                in("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdmittedPort, Port};

    #[test]
    fn admitted_ports_are_limited_to_early_boot_devices() {
        let serial_ports = [
            AdmittedPort::Data,
            AdmittedPort::InterruptEnable,
            AdmittedPort::FifoControl,
            AdmittedPort::LineControl,
            AdmittedPort::ModemControl,
            AdmittedPort::LineStatus,
        ];

        for port in serial_ports {
            let address = Port::new(port).admitted.address();
            assert!((0x3f8..=0x3fd).contains(&address));
        }

        let pic_ports = [
            AdmittedPort::PicMasterCommand,
            AdmittedPort::PicMasterData,
            AdmittedPort::PicSlaveCommand,
            AdmittedPort::PicSlaveData,
        ];

        assert_eq!(
            pic_ports.map(|port| Port::new(port).admitted.address()),
            [0x20, 0x21, 0xa0, 0xa1]
        );
    }
}
