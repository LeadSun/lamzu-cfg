mod atlantis;
mod checksum;

use binrw::{BinRead, BinWrite};
use hidapi::{HidApi, HidDevice};

// Currently only the Lamzu Atlantis Mini Pro is supported. The protocol may be
// similar in other Lamzu mice but needs testing.
const VENDOR_ID: u16 = 0x3554;
const SUPPORTED_PRODUCTS: [u16; 2] = [0xf50d, 0xf50f];
const REPORT_ID: u8 = 8;

/// Trait for types implementing both `BinRead` and `BinWrite`.
pub trait BinRw: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> {}

impl<T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()>> BinRw for T {}

/// Finds and connects to the first compatible HID device.
///
/// Matches based on vendor ID, product ID, and supported HID report ID.
pub fn first_compatible_device(api: &HidApi) -> crate::Result<HidDevice> {
    for device_info in api.device_list() {
        if device_info.vendor_id() == VENDOR_ID
            && SUPPORTED_PRODUCTS.contains(&device_info.product_id())
        {
            let device = device_info.open_device(&api)?;
            let mut report_descriptor = [0; hidapi::MAX_REPORT_DESCRIPTOR_SIZE];
            let len = device.get_report_descriptor(&mut report_descriptor)?;
            if has_report(&report_descriptor[..len], REPORT_ID) {
                return Ok(device);
            }
        }
    }
    Err(crate::Error::NoDevice)
}

/// Tests whether `report_descriptor` contains `report_id`.
///
/// Implements a basic USB HID report descriptor parser that skips any items
/// that are not report ID items. Returns `true` if any report ID item matches
/// `report_id`.
fn has_report(report_descriptor: &[u8], report_id: u8) -> bool {
    let mut i = 0;
    while i < report_descriptor.len() {
        let prefix = report_descriptor[i];
        i += 1;

        // Long item
        if prefix == 0b1111_1110 {
            unimplemented!("Long report descriptor item parsing is unimplemented");
        } else {
            // 1 byte report ID item
            if prefix == 0b1000_0101 {
                if report_descriptor[i] == report_id {
                    return true;
                }
                i += 1;
            } else {
                let data_len = match prefix & 0b11 {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    _ => unreachable!(),
                };

                // Skip item
                i += data_len;
            }
        }
    }

    false
}
