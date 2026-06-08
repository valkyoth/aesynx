use core::cell::Cell;
use core::fmt::{self, Write};
use core::marker::PhantomData;

use crate::port::{AdmittedPort, Port};

const DLAB: u8 = 0x80;
const EIGHT_BITS_NO_PARITY_ONE_STOP: u8 = 0x03;
const FIFO_ENABLE_CLEAR: u8 = 0xc7;
const MODEM_READY: u8 = 0x0b;
const TRANSMIT_EMPTY: u8 = 0x20;
const DIVISOR_LOW_38400_BAUD: u8 = 0x03;
const DIVISOR_HIGH_38400_BAUD: u8 = 0x00;
const MAX_TRANSMIT_POLLS: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Com1 {
    _single_core: PhantomData<Cell<()>>,
}

impl Com1 {
    pub const fn new() -> Self {
        Self {
            _single_core: PhantomData,
        }
    }

    pub fn init(self) {
        Port::new(AdmittedPort::InterruptEnable).write_u8(0x00);
        Port::new(AdmittedPort::LineControl).write_u8(DLAB);
        Port::new(AdmittedPort::Data).write_u8(DIVISOR_LOW_38400_BAUD);
        Port::new(AdmittedPort::InterruptEnable).write_u8(DIVISOR_HIGH_38400_BAUD);
        Port::new(AdmittedPort::LineControl).write_u8(EIGHT_BITS_NO_PARITY_ONE_STOP);
        Port::new(AdmittedPort::FifoControl).write_u8(FIFO_ENABLE_CLEAR);
        Port::new(AdmittedPort::ModemControl).write_u8(MODEM_READY);
    }

    pub fn write_byte(self, byte: u8) -> Result<(), SerialError> {
        let mut polls = 0u32;
        while Port::new(AdmittedPort::LineStatus).read_u8() & TRANSMIT_EMPTY == 0 {
            if polls >= MAX_TRANSMIT_POLLS {
                return Err(SerialError::TransmitTimeout);
            }
            polls += 1;
            core::hint::spin_loop();
        }

        Port::new(AdmittedPort::Data).write_u8(byte);
        Ok(())
    }

    pub fn write_ascii(self, value: &str) {
        for byte in value.bytes() {
            if byte == b'\n' {
                let _ = self.write_byte(b'\r');
            }
            let _ = self.write_byte(byte);
        }
    }
}

impl Default for Com1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for Com1 {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        (*self).write_ascii(value);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerialError {
    TransmitTimeout,
}

pub fn init() {
    Com1::new().init();
}

pub fn write_str(value: &str) {
    Com1::new().write_ascii(value);
}

pub fn write_fmt(args: fmt::Arguments<'_>) {
    let mut serial = Com1::new();
    let _ = serial.write_fmt(args);
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::write_fmt(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($format:expr) => {
        $crate::serial_print!(concat!($format, "\n"))
    };
    ($format:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($format, "\n"), $($arg)*)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn serial_timeout_bound_is_nonzero() {
        let max_polls = super::MAX_TRANSMIT_POLLS;
        assert!(max_polls > 0);
    }

    #[test]
    fn com1_is_single_core_marker_type() {
        let _serial = super::Com1::new();
        assert_eq!(core::mem::size_of::<super::Com1>(), 0);
    }
}
