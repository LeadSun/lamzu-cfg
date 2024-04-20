use std::io::{Read, Seek, SeekFrom, Write};

/// Wraps a reader / writer to add the calculation of a checksum.
///
/// The checksum is implemented as the wrapping subtraction of bytes from a
/// starting value of `85`.
pub struct Checksum<T> {
    inner: T,
    checksum: u8,
}

impl<T: Seek> Checksum<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            checksum: 85,
        }
    }

    pub fn checksum(&self) -> u8 {
        self.checksum
    }
}

impl<T: Read> Read for Checksum<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let rc = self.inner.read(buf);
        for byte in buf {
            self.checksum = self.checksum.wrapping_sub(*byte);
        }
        rc
    }
}

impl<T: Seek> Seek for Checksum<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let rc = self.inner.seek(pos)?;
        Ok(rc)
    }
}

impl<T: Write> Write for Checksum<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for byte in buf {
            self.checksum = self.checksum.wrapping_sub(*byte);
        }
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub fn compute_checksum(buf: &[u8]) -> u8 {
    let mut checksum: u8 = 85;
    for byte in buf {
        checksum = checksum.wrapping_sub(*byte);
    }

    checksum
}
