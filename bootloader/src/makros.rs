// Helper struct
pub use uguid::Guid;

macro_rules! uefi_println {
    ($con_out:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        // Helper struct
        pub struct Utf16Writer<'a> {
            buf: &'a mut [u16],
            cursor: &'a mut usize,
        }

        impl core::fmt::Write for Utf16Writer<'_> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                for c in s.encode_utf16() {
                    if *self.cursor >= self.buf.len() {
                        return Err(core::fmt::Error);
                    }
                    self.buf[*self.cursor] = c;
                    *self.cursor += 1;
                }
                Ok(())
            }
        }

        let mut buf = [0u16; 512]; // 512 UTF-16 characters
        let mut cursor = 0;

        let _ = write!(&mut Utf16Writer { buf: &mut buf, cursor: &mut cursor }, $($arg)*);

        // Add CRLF manually
        if cursor + 2 < buf.len() {
            buf[cursor] = '\r' as u16;
            cursor += 1;
            buf[cursor] = '\n' as u16;
            cursor += 1;
        }

        // Null-terminate
        if cursor < buf.len() {
            buf[cursor] = 0;
        }


        ((*$con_out).output_string)($con_out, buf.as_mut_ptr());
    }};
}

pub(crate) use uefi_println;

//---------------------------

#[macro_export]
macro_rules! guid {
    ($l:expr, $m:expr, $h:expr, $b1:expr, $b2:expr, $($rest:expr),*) => {
        Guid {
            time_low: [
                ($l >>  0) as u8,
                ($l >>  8) as u8,
                ($l >> 16) as u8,
                ($l >> 24) as u8,
            ],
            time_mid: [
                ($m >> 0) as u8,
                ($m >> 8) as u8,
            ],
            time_hi_and_version: [
                ($h >> 0) as u8,
                ($h >> 8) as u8,
            ],
            clk_seq_hi_res: $b1,
            clk_seq_low: $b2,
            node: [ $($rest),* ],
        }
    };
}

pub(crate) use guid;
