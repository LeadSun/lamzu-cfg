use hidapi::{HidApi, HidDevice, HidResult};
use serde::Serialize;
use std::fmt;

// Currently only the Lamzu Atlantis Mini Pro is supported. The protocol may be
// similar in other Lamzu mice but needs testing.
const VENDOR_ID: u16 = 0x3554;
const REPORT_ID: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Product {
    AtlantisWired,
    AtlantisWireless1K,
    AtlantisWireless4K,
    Unknown,
}

impl Product {
    pub fn from_usb_product(product_id: u16) -> Product {
        match product_id {
            0xf50f => Self::AtlantisWired,
            0xf50d => Self::AtlantisWireless1K,
            0xf510 => Self::AtlantisWireless4K,
            _ => Self::Unknown,
        }
    }

    pub fn max_poll_rate(&self) -> u16 {
        match self {
            Self::AtlantisWired => 1000,
            Self::AtlantisWireless1K => 1000,
            Self::AtlantisWireless4K => 4000,
            Self::Unknown => 1000,
        }
    }
}

impl Default for Product {
    fn default() -> Self {
        Self::AtlantisWireless4K
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AtlantisWired => write!(f, "Lamzu Atlantis Wired"),
            Self::AtlantisWireless1K => write!(f, "Lamzu Atlantis Wireless (1K)"),
            Self::AtlantisWireless4K => write!(f, "Lamzu Atlantis Wireless (4K)"),
            Self::Unknown => write!(f, "Unknown Device"),
        }
    }
}

/// Lists potentially compatible devices with their detected products.
pub fn devices() -> HidResult<Vec<(HidDevice, Product)>> {
    get_devices(None)
}

/// Lists potentially compatible devices with their detected products, filtered
/// by USB product ID.
pub fn devices_by_pid(pid: u16) -> HidResult<Vec<(HidDevice, Product)>> {
    get_devices(Some(pid))
}

fn get_devices(filter_pid: Option<u16>) -> HidResult<Vec<(HidDevice, Product)>> {
    let mut api = HidApi::new()?;
    api.reset_devices()?;
    api.add_devices(VENDOR_ID, filter_pid.unwrap_or(0))?;
    let mut device_infos: Vec<_> = api.device_list().collect();

    // Deduplicate based on hidraw path.
    device_infos.sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());
    device_infos.dedup_by(|a, b| a.path() == b.path());

    let mut devices: Vec<_> = device_infos
        .iter()
        .filter_map(|info| {
            let device = info
                .open_device(&api)
                .inspect_err(|e| eprintln!("USB HID error: {e}"))
                .ok()?;
            let id = identify(&device)
                .inspect_err(|e| eprintln!("USB HID error: {e}"))
                .ok()??;
            Some((device, id))
        })
        .collect();

    // Sort by connection priority.
    devices.sort_by(|(_, product_a), (_, product_b)| product_a.cmp(&product_b));

    Ok(devices)
}

/// Attempt to identify the connected device, returning `None` for devices that
/// are incompatible.
pub fn identify(device: &HidDevice) -> HidResult<Option<Product>> {
    let device_info = device.get_device_info()?;
    if device_info.vendor_id() == VENDOR_ID {
        let mut report_descriptor = [0; hidapi::MAX_REPORT_DESCRIPTOR_SIZE];
        let desc_len = device.get_report_descriptor(&mut report_descriptor)?;
        if has_report(&report_descriptor[..desc_len], REPORT_ID) {
            return Ok(Some(Product::from_usb_product(device_info.product_id())));
        }
    }
    Ok(None)
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
