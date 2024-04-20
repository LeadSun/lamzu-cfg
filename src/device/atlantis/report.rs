use crate::device::atlantis::Checksum;
use binrw::{
    binrw,
    meta::{ReadEndian, WriteEndian},
    BinRead, BinWrite,
};
use hidapi::HidDevice;
use std::io::Cursor;

/// For all USB HID reports.
pub trait Report {
    /// HID report ID.
    const REPORT_ID: u8;

    /// Fixed, all-inclusive size of the report in bytes.
    const SIZE: usize;
}

/// The standard report used for both requests and responses.
#[binrw]
#[brw(big, stream = s, map_stream = Checksum::new)]
#[derive(Debug, Clone)]
pub struct StandardReport {
    // Attach report ID (`magic`) here so it's included in the checksum.
    #[brw(magic = 8u8)]
    cmd: Command,

    /// Error code received from mouse should be zero for ok.
    error: u8,

    /// Address to read data from / write data to in mouse storage. Seems to only
    /// be used for `ReadProfileData` and `WriteProfileData` commands.
    address: u16,

    #[br(temp)]
    #[bw(try_calc(u8::try_from(data.len())))]
    len: u8,

    #[br(count = len)]
    #[brw(pad_size_to = 10, assert(data.len() <= 10))]
    data: Vec<u8>,

    #[br(temp, assert(s.checksum() == 0, "Bad checksum"))]
    #[bw(calc(s.checksum()))]
    _checksum: u8,
}

impl StandardReport {
    /// Constructs a report for requesting to read `length` bytes of data from
    /// the active profile at `address`.
    pub fn read_profile_data(address: usize, length: usize) -> Self {
        Self {
            cmd: Command::ReadProfileData,
            error: 0,
            address: address as u16,
            data: vec![0; length as usize],
        }
    }

    /// Constructs a report for writing `data` to the active profile at
    /// `address`.
    pub fn write_profile_data(address: usize, data: Vec<u8>) -> Self {
        Self {
            cmd: Command::WriteProfileData,
            error: 0,
            address: address as u16,
            data,
        }
    }

    /// Constructs a report for requesting the index of the active profile.
    pub fn read_active_profile() -> Self {
        Self {
            cmd: Command::ReadActiveProfile,
            error: 0,
            address: 0,
            data: Vec::new(),
        }
    }

    /// Constructs a report for setting the index of the active profile.
    pub fn write_active_profile(profile_index: u8) -> Self {
        Self {
            cmd: Command::WriteActiveProfile,
            error: 0,
            address: 0,
            data: vec![profile_index],
        }
    }

    /// Returns a reference to the internal data unless the report indicates an
    /// error.
    pub fn data(&self) -> crate::Result<&[u8]> {
        if self.error == 0 {
            Ok(&self.data)
        } else {
            Err(crate::Error::MouseErrorResponse(self.error))
        }
    }

    /// Consumes `self` and returns the internal data unless the report
    /// indicates an error.
    pub fn into_data(self) -> crate::Result<Vec<u8>> {
        if self.error == 0 {
            Ok(self.data)
        } else {
            Err(crate::Error::MouseErrorResponse(self.error))
        }
    }

    /// Returns whether this report could be a response to the `other` report.
    pub fn is_valid_response_for(&self, other: &Self) -> bool {
        // Response should have the same command type as the request.
        self.cmd == other.cmd
    }
}

impl Report for StandardReport {
    const REPORT_ID: u8 = 8;
    const SIZE: usize = 17;
}

#[binrw]
#[brw(big, repr = u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    /// Write profile data to address within active profile.
    WriteProfileData = 7,

    /// Read profile data from address within active profile.
    ReadProfileData = 8,

    /// Read index of active profile.
    ReadActiveProfile = 14,

    /// Write index of active profile.
    WriteActiveProfile = 15,
}

/// Reads a report from the device and attempts to deserialize it as `R`.
///
/// Returns `Error::UnexpectedReport` if the received report has the wrong ID.
pub fn read_report<A, R>(device: &HidDevice) -> crate::Result<R>
where
    A: Default,
    for<'a> R: Report + BinRead<Args<'a> = A> + ReadEndian,
{
    let mut report_bytes = vec![0; R::SIZE];
    let read_bytes = device.read(&mut report_bytes)?;
    if report_bytes[0] != R::REPORT_ID {
        return Err(crate::Error::UnexpectedReport);
    }

    assert!(read_bytes == R::SIZE);
    let mut cursor = Cursor::new(report_bytes);
    Ok(R::read(&mut cursor)?)
}

/// Serializes and writes `report` to the device.
pub fn write_report<A, R>(device: &HidDevice, report: &R) -> crate::Result<()>
where
    A: Default,
    for<'a> R: Report + BinWrite<Args<'a> = A> + WriteEndian,
{
    let mut report_bytes = Cursor::new(Vec::new());
    report.write(&mut report_bytes)?;
    assert!(report_bytes.get_ref().len() == R::SIZE);
    assert!(device.write(report_bytes.get_ref())? == R::SIZE);
    Ok(())
}

/// Writes a report to the device and attempts to read a matching response.
pub fn make_request(device: &HidDevice, request: &StandardReport) -> crate::Result<StandardReport> {
    write_report(device, request)?;

    // A request may result in multiple responses so skip the unwanted ones.
    for _ in 0..3 {
        match read_report::<_, StandardReport>(device) {
            Ok(response) => {
                if response.is_valid_response_for(&request) {
                    return Ok(response);
                }
            }
            Err(crate::Error::UnexpectedReport) => {}
            result => return result,
        }
    }

    Err(crate::Error::NoResponse)
}
