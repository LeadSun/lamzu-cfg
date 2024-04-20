use binrw::{binrw, BinRead, BinWrite};
use std::fmt;
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

pub trait Checksummable: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> {}

impl<T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()>> Checksummable for T {}

/// Wraps an object to add a calculated checksum to the end.
#[binrw]
#[brw(big, stream = s, map_stream = Checksum::new)]
pub struct Checksummed<T: Checksummable> {
    inner: T,

    #[br(temp, assert(s.checksum() == 0, "Bad checksum"))]
    #[bw(calc(s.checksum()))]
    _checksum: u8,
}

impl<T: Checksummable> Checksummed<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Checksummable> From<T> for Checksummed<T> {
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Checksummable + Default> Default for Checksummed<T> {
    fn default() -> Self {
        Self {
            inner: T::default(),
        }
    }
}

impl<T: Checksummable + Clone> Clone for Checksummed<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Checksummable + fmt::Debug> fmt::Debug for Checksummed<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}
