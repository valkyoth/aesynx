use core::fmt::{self, Write};

pub const MAX_PANIC_FIELD_OUTPUT_BYTES: usize = 256;

pub fn write_panic_location(
    output: &mut impl fmt::Write,
    file: &str,
    line: u32,
    column: u32,
) -> fmt::Result {
    output.write_str("panic location=")?;
    write_escaped_field(output, file_basename(file))?;
    writeln!(output, " line={line} column={column}")
}

pub fn write_panic_message(output: &mut impl fmt::Write, args: fmt::Arguments<'_>) -> fmt::Result {
    output.write_str("panic message=")?;
    let truncated = {
        let mut message = EscapedPanicFieldWriter::new(output);
        fmt::write(&mut message, args)?;
        message.truncated()
    };

    if truncated {
        output.write_str("...<truncated>")?;
    }

    output.write_char('\n')
}

fn write_escaped_field(output: &mut impl fmt::Write, value: &str) -> fmt::Result {
    let truncated = {
        let mut field = EscapedPanicFieldWriter::new(output);
        field.write_str(value)?;
        field.truncated()
    };

    if truncated {
        output.write_str("...<truncated>")?;
    }

    Ok(())
}

fn file_basename(path: &str) -> &str {
    let path = path
        .rsplit_once('/')
        .map_or(path, |_prefix_and_name| _prefix_and_name.1);
    path.rsplit_once('\\')
        .map_or(path, |_prefix_and_name| _prefix_and_name.1)
}

struct EscapedPanicFieldWriter<'a, W: fmt::Write + ?Sized> {
    output: &'a mut W,
    remaining: usize,
    truncated: bool,
}

impl<'a, W: fmt::Write + ?Sized> EscapedPanicFieldWriter<'a, W> {
    fn new(output: &'a mut W) -> Self {
        Self {
            output,
            remaining: MAX_PANIC_FIELD_OUTPUT_BYTES,
            truncated: false,
        }
    }

    fn truncated(&self) -> bool {
        self.truncated
    }

    fn write_ascii_byte(&mut self, byte: u8) -> fmt::Result {
        if self.remaining == 0 {
            self.truncated = true;
            return Ok(());
        }

        self.output.write_char(char::from(byte))?;
        self.remaining -= 1;
        Ok(())
    }

    fn write_fragment(&mut self, fragment: &str) -> fmt::Result {
        if fragment.len() > self.remaining {
            self.truncated = true;
            self.remaining = 0;
            return Ok(());
        }

        self.output.write_str(fragment)?;
        self.remaining -= fragment.len();
        Ok(())
    }
}

impl<W: fmt::Write + ?Sized> fmt::Write for EscapedPanicFieldWriter<'_, W> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for byte in value.bytes() {
            match byte {
                b'\n' => self.write_fragment("\\n")?,
                b'\r' => self.write_fragment("\\r")?,
                b'\t' => self.write_fragment("\\t")?,
                b'\\' => self.write_fragment("\\\\")?,
                b'[' => self.write_fragment("\\[")?,
                b']' => self.write_fragment("\\]")?,
                0x20..=0x7e => self.write_ascii_byte(byte)?,
                _non_ascii_or_control => self.write_fragment("?")?,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::{self, Write};

    use super::{
        MAX_PANIC_FIELD_OUTPUT_BYTES, file_basename, write_panic_location, write_panic_message,
    };

    #[test]
    fn panic_location_uses_escaped_basename() {
        let mut output = FixedBuf::default();

        assert_eq!(
            write_panic_location(&mut output, "/build/root/src/[main]\n.rs", 7, 9),
            Ok(())
        );
        assert_eq!(
            output.as_str(),
            "panic location=\\[main\\]\\n.rs line=7 column=9\n"
        );
    }

    #[test]
    fn panic_location_handles_windows_paths() {
        assert_eq!(file_basename(r"C:\work\aesynx\src\main.rs"), "main.rs");
    }

    #[test]
    fn panic_message_writer_escapes_record_injection() {
        let mut output = FixedBuf::default();

        assert_eq!(
            write_panic_message(
                &mut output,
                format_args!("fatal\n[core=7][phase=panic][kernel][FATAL]")
            ),
            Ok(())
        );
        assert_eq!(
            output.as_str(),
            "panic message=fatal\\n\\[core=7\\]\\[phase=panic\\]\\[kernel\\]\\[FATAL\\]\n"
        );
    }

    #[test]
    fn panic_message_writer_bounds_output() {
        let mut output = FixedBuf::default();
        let expected = FixedBuf::repeat('a', MAX_PANIC_FIELD_OUTPUT_BYTES);

        assert_eq!(
            write_panic_message(
                &mut output,
                format_args!(
                    "{}{}",
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                     aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                     aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
                     aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                )
            ),
            Ok(())
        );
        let output = output.as_str();
        let payload_start = "panic message=".len();
        let payload_end = payload_start + MAX_PANIC_FIELD_OUTPUT_BYTES;

        assert!(output.starts_with("panic message="));
        assert_eq!(&output[payload_start..payload_end], expected.as_str());
        assert_eq!(&output[payload_end..], "...<truncated>\n");
    }

    struct FixedBuf {
        bytes: [u8; 512],
        len: usize,
    }

    impl Default for FixedBuf {
        fn default() -> Self {
            Self {
                bytes: [0; 512],
                len: 0,
            }
        }
    }

    impl FixedBuf {
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
        }

        fn repeat(value: char, count: usize) -> Self {
            let mut output = Self::default();
            for _index in 0..count {
                let _ = output.write_char(value);
            }
            output
        }
    }

    impl Write for FixedBuf {
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
