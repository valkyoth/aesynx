use core::fmt::{self, Write};

use crate::port::Port;

const COM1_BASE: u16 = 0x3f8;
const INTERRUPT_ENABLE: u16 = COM1_BASE + 1;
const FIFO_CONTROL: u16 = COM1_BASE + 2;
const LINE_CONTROL: u16 = COM1_BASE + 3;
const MODEM_CONTROL: u16 = COM1_BASE + 4;
const LINE_STATUS: u16 = COM1_BASE + 5;

const DLAB: u8 = 0x80;
const EIGHT_BITS_NO_PARITY_ONE_STOP: u8 = 0x03;
const FIFO_ENABLE_CLEAR: u8 = 0xc7;
const MODEM_READY: u8 = 0x0b;
const TRANSMIT_EMPTY: u8 = 0x20;
const DIVISOR_LOW_38400_BAUD: u8 = 0x03;
const DIVISOR_HIGH_38400_BAUD: u8 = 0x00;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Com1;

impl Com1 {
    pub fn init(self) {
        Port::new(INTERRUPT_ENABLE).write_u8(0x00);
        Port::new(LINE_CONTROL).write_u8(DLAB);
        Port::new(COM1_BASE).write_u8(DIVISOR_LOW_38400_BAUD);
        Port::new(INTERRUPT_ENABLE).write_u8(DIVISOR_HIGH_38400_BAUD);
        Port::new(LINE_CONTROL).write_u8(EIGHT_BITS_NO_PARITY_ONE_STOP);
        Port::new(FIFO_CONTROL).write_u8(FIFO_ENABLE_CLEAR);
        Port::new(MODEM_CONTROL).write_u8(MODEM_READY);
    }

    pub fn write_byte(self, byte: u8) {
        while Port::new(LINE_STATUS).read_u8() & TRANSMIT_EMPTY == 0 {
            core::hint::spin_loop();
        }

        Port::new(COM1_BASE).write_u8(byte);
    }

    pub fn write_ascii(self, value: &str) {
        for byte in value.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
    }
}

impl Write for Com1 {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        (*self).write_ascii(value);
        Ok(())
    }
}

pub fn init() {
    Com1.init();
}

pub fn write_str(value: &str) {
    Com1.write_ascii(value);
}

pub fn write_fmt(args: fmt::Arguments<'_>) {
    let mut serial = Com1;
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
    fn serial_port_constants_use_legacy_com1() {
        assert_eq!(super::COM1_BASE, 0x3f8);
        assert_eq!(super::LINE_STATUS, 0x3fd);
    }
}
