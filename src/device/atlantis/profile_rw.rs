//! Reader and writer for Lamzu Atlantis profile data.

use crate::device::atlantis::{make_request, StandardReport};
use hidapi::HidDevice;
use std::io::{self, Read, Seek, SeekFrom, Write};

/// No more data at / after this address.
const DATA_END: usize = 0x1b00;

/// A buffered reader that requests profile data from the device as needed.
pub struct ProfileReader<'a> {
    /// HID device to read from.
    device: &'a HidDevice,

    /// Buffered bytes read from `device` starting at `position`.
    buf: Vec<u8>,

    /// The origin address where data will start being read from.
    address: usize,

    /// The cursor position relative to `address` where data will be read from.
    position: usize,
}

impl<'a> ProfileReader<'a> {
    pub fn new(device: &'a HidDevice, address: usize) -> Self {
        Self {
            device,
            buf: Vec::new(),
            address,
            position: 0,
        }
    }
}

impl<'a> Read for ProfileReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buf.is_empty() {
            // Don't read past the end of the data.
            let len = (DATA_END - (self.address + self.position)).min(10);
            if len == 0 {
                return Ok(0);
            }

            self.buf = make_request(
                self.device,
                &StandardReport::read_profile_data(self.address + self.position, len),
            )
            .and_then(|response| response.into_data())
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        }

        let clone_bytes = buf.len().min(self.buf.len());
        buf.clone_from_slice(&self.buf[..clone_bytes]);
        self.buf.drain(..clone_bytes);
        self.position += clone_bytes;
        Ok(clone_bytes)
    }
}

impl<'a> Seek for ProfileReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(from_start) => {
                self.buf.clear();
                self.position = from_start as usize;
            }
            SeekFrom::End(from_end) => {
                self.buf.clear();
                self.position = DATA_END
                    .checked_add_signed(from_end as isize)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?
            }
            SeekFrom::Current(from_current) => {
                // Attempt to retain some buffer contents.
                if from_current < 0 || (from_current as usize) >= self.buf.len() {
                    self.buf.clear();
                } else if from_current > 0 {
                    self.buf.drain(..(from_current as usize));
                }

                self.position = self
                    .position
                    .checked_add_signed(from_current as isize)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?
            }
        }
        Ok(self.position as u64)
    }
}

/// A buffered writer that sends profile data to the device when the buffer
/// fills.
pub struct ProfileWriter<'a> {
    /// HID device to write to.
    device: &'a HidDevice,

    /// Buffered bytes to be written to `device` starting at `position`.
    buf: Vec<u8>,

    /// The origin address where data will start being written to.
    address: usize,

    /// The cursor position relative to `address` where data will be written to.
    position: usize,
}

impl<'a> ProfileWriter<'a> {
    pub fn new(device: &'a HidDevice, address: usize) -> Self {
        Self {
            device,
            buf: Vec::new(),
            address,
            position: 0,
        }
    }

    /// Writes a single report containing up to 10 bytes of buffered data.
    fn write_report(&mut self) -> io::Result<usize> {
        // Don't write past the end of the data.
        let len = (DATA_END - (self.address + self.position))
            .min(10)
            .min(self.buf.len());
        if len == 0 {
            return Ok(0);
        }

        make_request(
            self.device,
            &StandardReport::write_profile_data(
                self.address + self.position,
                self.buf[..len].to_vec(),
            ),
        )
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        self.position += len;
        self.buf.drain(..len);
        Ok(len)
    }
}

impl<'a> Write for ProfileWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(&buf);
        if self.buf.len() < 10 {
            return Ok(buf.len());
        }

        self.write_report()
    }

    fn flush(&mut self) -> io::Result<()> {
        while self.buf.len() > 0 {
            if let Ok(0) = self.write_report() {
                return Ok(());
            }
        }
        Ok(())
    }
}

impl<'a> Seek for ProfileWriter<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(from_start) => {
                self.flush()?;
                self.position = from_start as usize;
            }
            SeekFrom::End(from_end) => {
                self.flush()?;
                self.position = DATA_END
                    .checked_add_signed(from_end as isize)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?
            }
            SeekFrom::Current(from_current) => {
                if from_current != 0 {
                    self.flush()?;
                }
                self.position = self
                    .position
                    .checked_add_signed(from_current as isize)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?
            }
        }
        Ok(self.position as u64)
    }
}

impl<'a> Drop for ProfileWriter<'a> {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}
