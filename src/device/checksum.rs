use crate::device::BinRw;
use binrw::binrw;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;

/// Wraps a reader / writer to add the calculation of a checksum.
pub struct Stream<S, A: Algorithm> {
    /// Wrapped stream.
    inner: S,

    /// Checksum calculated over read / written bytes.
    checksum: A,

    /// Position of next byte to hash.
    position: u64,
}

impl<S: Seek, A: Algorithm + Default> Stream<S, A> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            checksum: A::default(),
            position: 0,
        }
    }

    pub fn checksum(&self) -> &A {
        &self.checksum
    }
}

impl<S: Read + Seek, A: Algorithm> Read for Stream<S, A> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let position = self.inner.stream_position()?;
        let size = self.inner.read(buf)?;

        // Make sure that read bytes aren't checksummed more than once.
        for (i, byte) in buf.iter().enumerate() {
            if position + i as u64 >= self.position {
                self.checksum.write(&[*byte]);
            }
        }
        self.position = position + size as u64;
        Ok(size)
    }
}

impl<S: Seek, A: Algorithm> Seek for Stream<S, A> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_position = self.inner.seek(pos)?;
        Ok(new_position)
    }
}

impl<S: Write, A: Algorithm> Write for Stream<S, A> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for byte in buf {
            self.checksum.write(&[*byte]);
        }
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// A trait for data integrity checksum algorithms.
pub trait Algorithm {
    type Output;

    /// Adds bytes to the checksum.
    fn write(&mut self, bytes: &[u8]);

    /// Returns the finished checksum.
    fn finish(&self) -> Self::Output;

    /// Returns whether the written data is valid.
    ///
    /// Make sure that the received checksum value has been written before
    /// calling.
    fn is_valid(&self) -> bool;
}

/// A trait for 8 bit checksum algorithms.
pub trait Algorithm8: Algorithm<Output = u8> {}

impl<T: Algorithm<Output = u8>> Algorithm8 for T {}

/// 8 bit sum complement (two's complement) checksum with an initial value.
#[derive(Debug, Clone)]
pub struct SumComplement8<const INIT: u8> {
    sum: u8,
}

impl<const INIT: u8> Default for SumComplement8<INIT> {
    fn default() -> Self {
        Self { sum: INIT }
    }
}

impl<const INIT: u8> Algorithm for SumComplement8<INIT> {
    type Output = u8;

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.sum = self.sum.wrapping_add(*byte);
        }
    }

    fn finish(&self) -> Self::Output {
        // Two's complement
        0u8.wrapping_sub(self.sum)
    }

    fn is_valid(&self) -> bool {
        self.finish() == 0
    }
}

/// Wraps an object to add a calculated checksum to the end.
#[binrw]
#[brw(stream = s, map_stream = Stream::<_, A>::new)]
pub struct Append<T: BinRw, A: Algorithm<Output = O> + Default, O: BinRw> {
    inner: T,

    #[br(temp, assert(s.checksum().is_valid(), "Bad checksum"))]
    #[bw(calc(s.checksum().finish()))]
    _checksum: O,

    phantom: PhantomData<A>,
}

impl<T: BinRw, A: Algorithm<Output = O> + Default, O: BinRw> Append<T, A, O> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// Wraps an object to add a calculated 8 bit checksum to the end.
pub type Append8<T, A> = Append<T, A, u8>;
