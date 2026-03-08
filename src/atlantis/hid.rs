use super::checksum;
use hidapi::HidDevice;

const REPORT_ID: u8 = 8;

pub struct Report {
    cmd: Command,
    error: u8,
    address: u16,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    ReadBatteryVoltage = 4,

    // Read / write flash for active profile.
    WriteFlash = 7,
    ReadFlash = 8,

    ReadActiveProfile = 14,
    WriteActiveProfile = 15,
}

impl Command {
    fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            4 => Some(Self::ReadBatteryVoltage),
            7 => Some(Self::WriteFlash),
            8 => Some(Self::ReadFlash),
            14 => Some(Self::ReadActiveProfile),
            15 => Some(Self::WriteActiveProfile),
            _ => None,
        }
    }
}

pub fn read_battery_voltage(device: &HidDevice) -> crate::Result<u16> {
    let report = Report {
        cmd: Command::ReadBatteryVoltage,
        error: 0,
        address: 0,
        payload: Vec::new(),
    };
    let response = make_request(device, &report, true)?;
    Ok(u16::from_be_bytes([
        response.payload[2],
        response.payload[3],
    ]))
}

pub fn read_flash(device: &HidDevice, address: usize, length: usize) -> crate::Result<Vec<u8>> {
    let mut data = Vec::new();
    while data.len() < length {
        let report = Report {
            cmd: Command::ReadFlash,
            error: 0,
            address: (address + data.len()) as u16,
            payload: vec![0; (length - data.len()).min(10)],
        };
        data.append(&mut make_request(device, &report, false).map(|response| response.payload)?);
    }
    Ok(data)
}

pub fn write_flash(device: &HidDevice, mut address: usize, mut data: Vec<u8>) -> crate::Result<()> {
    while data.len() > 0 {
        let payload: Vec<u8> = data.drain(..data.len().min(10)).collect();
        let len = payload.len();
        let report = Report {
            cmd: Command::WriteFlash,
            error: 0,
            address: address as u16,
            payload,
        };
        make_request(device, &report, false).map(|_| ())?;
        address += len;
    }
    Ok(())
}

pub fn read_active_profile(device: &HidDevice) -> crate::Result<u8> {
    let report = Report {
        cmd: Command::ReadActiveProfile,
        error: 0,
        address: 0,
        payload: vec![],
    };
    make_request(device, &report, true).map(|response| response.payload[0])
}

pub fn write_active_profile(device: &HidDevice, profile_index: u8) -> crate::Result<()> {
    let report = Report {
        cmd: Command::WriteActiveProfile,
        error: 0,
        address: 0,
        payload: vec![profile_index],
    };
    make_request(device, &report, false).map(|_| ())
}

/// Writes a report to the device and attempts to read a matching response.
fn make_request(device: &HidDevice, request: &Report, ignore_len: bool) -> crate::Result<Report> {
    write_report(device, request)?;

    // A request may result in multiple responses so skip the unwanted ones.
    for _ in 0..3 {
        match read_report(device, ignore_len) {
            Ok(Some(response)) => {
                if response.cmd == request.cmd
                    && (ignore_len || response.payload.len() == request.payload.len())
                {
                    if response.error == 0 {
                        return Ok(response);
                    } else {
                        return Err(crate::Error::MouseErrorResponse(response.error));
                    }
                }
            }
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    Err(crate::Error::NoResponse)
}

fn read_report(device: &HidDevice, ignore_len: bool) -> crate::Result<Option<Report>> {
    let mut buf = vec![0; 17];
    let len = device.read(&mut buf)?;
    if buf[0] != REPORT_ID {
        // Wrong report.
        return Ok(None);
    }
    assert!(len == 17);
    assert!(checksum(&buf[..16]) == buf[16]);
    let payload_len = if ignore_len { 10 } else { buf[5] as usize };
    let Some(cmd) = Command::from_u8(buf[1]) else {
        // Response for unknown command.
        return Ok(None);
    };
    Ok(Some(Report {
        cmd,
        error: buf[2],
        address: u16::from_be_bytes([buf[3], buf[4]]),
        payload: buf[6..(6 + payload_len)].to_vec(),
    }))
}

fn write_report(device: &HidDevice, report: &Report) -> crate::Result<()> {
    assert!(report.payload.len() <= 10);
    let mut buf = vec![REPORT_ID, report.cmd as u8, report.error];
    buf.extend(u16::to_be_bytes(report.address));
    buf.push(report.payload.len() as u8);
    buf.extend(&report.payload);
    buf.resize(16, 0);
    buf.push(checksum(&buf));
    assert!(device.write(&buf)? == 17);

    Ok(())
}
