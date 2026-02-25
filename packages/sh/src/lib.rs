//! Semihosting

// increasing counters while writing them to disk sounds racy so disable coverage here
#![cfg_attr(feature = "codecov", feature(coverage_attribute))]
#![cfg_attr(feature = "codecov", coverage(off))]
#![no_std]

pub use semihosting::println;

/// Exits the application
pub fn exit() -> ! {
    #[cfg(feature = "codecov")]
    {
        use core::fmt::Write as _;
        use core::sync::atomic::{self, AtomicBool};

        use semihosting::experimental::random;
        use semihosting::fs::{self, File};

        use crate::string::String;
        use crate::writer::Writer;

        static ONCE: AtomicBool = AtomicBool::new(false);

        // FIXME this needs to be a Mutex or a second thread can exit the
        // process while this is ongoing
        if !ONCE.swap(true, atomic::Ordering::Relaxed) {
            let mut bytes = [0; 8];

            if random::fill_bytes(&mut bytes).is_ok() {
                let id = u64::from_le_bytes(bytes);
                let mut path = String::default();

                _ = write!(path, "{id}.profraw");
                if let Ok(f) = File::create(&path) {
                    let mut writer = Writer::new(f);

                    if unsafe { minicov::capture_coverage(&mut writer).is_err() } {
                        _ = fs::remove_file(path);
                    } else {
                        writer.close()
                    }
                }
            }
        }
    }

    semihosting::process::exit(0)
}

#[cfg(feature = "codecov")]
mod writer {
    use minicov::{CoverageWriteError, CoverageWriter};
    use semihosting::fs::File;
    use semihosting::io::Write as _;

    pub struct Writer {
        f: File,
    }

    impl Writer {
        pub fn new(f: File) -> Self {
            Self { f }
        }

        pub fn close(mut self) {
            _ = self.f.flush();
        }
    }

    impl CoverageWriter for Writer {
        fn write(&mut self, bytes: &[u8]) -> Result<(), CoverageWriteError> {
            self.f.write_all(bytes).map_err(|_| CoverageWriteError)
        }
    }
}

#[cfg(feature = "codecov")]
mod string {
    use core::ffi::CStr;
    use core::fmt;

    #[derive(Default)]
    pub struct String {
        buf: [u8; 32],
        len: usize,
    }

    impl AsRef<CStr> for String {
        fn as_ref(&self) -> &CStr {
            CStr::from_bytes_with_nul(&self.buf[..self.len + 1]).unwrap()
        }
    }

    impl String {
        fn capacity(&self) -> usize {
            self.buf.len() - self.len - 1
        }
    }

    impl fmt::Write for String {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let len = s.len().min(self.capacity());
            self.buf[self.len..][..len].copy_from_slice(&s.as_bytes()[..len]);
            self.len += len;
            Ok(())
        }
    }
}
