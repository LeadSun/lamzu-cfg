mod atlantis;
pub use atlantis::Atlantis;
mod checksum;

use crate::Profile;
use binrw::{BinRead, BinWrite};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::fmt;

// Currently only the Lamzu Atlantis Mini Pro is supported. The protocol may be
// similar in other Lamzu mice but needs testing.
const VENDOR_ID: u16 = 0x3554;
const REPORT_ID: u8 = 8;

#[derive(Debug, Clone, Copy)]
pub enum Product {
    AtlantisWireless1K,
    AtlantisWireless4K,
    AtlantisWired,
}

impl Product {
    pub fn from_usb_product(product_id: u16) -> Option<Product> {
        match product_id {
            0xf50d => Some(Self::AtlantisWireless1K),
            0xf510 => Some(Self::AtlantisWireless4K),
            0xf50f => Some(Self::AtlantisWired),
            _ => None,
        }
    }

    pub fn max_poll_rate(&self) -> u16 {
        match self {
            Self::AtlantisWireless1K => 1000,
            Self::AtlantisWireless4K => 4000,
            Self::AtlantisWired => 1000,
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
            Self::AtlantisWireless1K => write!(f, "Lamzu Atlantis Wireless (1K)"),
            Self::AtlantisWireless4K => write!(f, "Lamzu Atlantis Wireless (4K)"),
            Self::AtlantisWired => write!(f, "Lamzu Atlantis Wired"),
        }
    }
}

/// Trait for supported mice that can be configured via profiles.
pub trait Mouse {
    /// Returns a specific profile from the device.
    fn profile(&self, device: &HidDevice, index: usize) -> crate::Result<Profile>;

    /// Write to a specific profile on the device.
    fn set_profile(&self, device: &HidDevice, index: usize, profile: &Profile)
        -> crate::Result<()>;

    /// Returns all profiles from the device.
    fn profiles(&self, device: &HidDevice) -> crate::Result<Vec<Profile>>;

    /// Write multiple profiles to the device.
    fn set_profiles(&self, device: &HidDevice, profiles: &[Profile]) -> crate::Result<()>;

    /// Returns the index of the currently active profile.
    fn active_profile_index(&self, device: &HidDevice) -> crate::Result<usize>;

    /// Set the active profile by index.
    fn set_active_profile_index(&self, device: &HidDevice, index: usize) -> crate::Result<()>;
}

/// Trait for types implementing both `BinRead` and `BinWrite`.
pub trait BinRw: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> {}

impl<T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()>> BinRw for T {}

/// HID device compatibility with this library.
#[derive(Debug)]
pub enum Compatibility {
    /// Device has correct vendor ID and report descriptor, and devices with
    /// this product ID have been tested to work.
    Tested(HidDevice, Product),

    /// Device has correct vendor ID and report descriptor, but devices with
    /// this product ID have not been tested. Use at your own risk.
    Untested(HidDevice),

    /// Device has incorrect vendor ID or report descriptor.
    Incompatible(DeviceInfo),
}

/// Returns `Compatibility` for each detected HID device.
pub fn device_compatibility(api: &HidApi) -> Vec<Compatibility> {
    let mut device_infos = api.device_list().collect::<Vec<_>>();

    // Deduplicate based on hidraw path.
    device_infos.sort_by(|a, b| a.path().partial_cmp(b.path()).unwrap());
    device_infos.dedup_by(|a, b| a.path() == b.path());

    device_infos
        .into_iter()
        .cloned()
        .map(|device_info| {
            if device_info.vendor_id() == VENDOR_ID {
                let mut report_descriptor = [0; hidapi::MAX_REPORT_DESCRIPTOR_SIZE];
                match device_info.open_device(&api).and_then(|device| {
                    device
                        .get_report_descriptor(&mut report_descriptor)
                        .map(|len| (device, len))
                }) {
                    Ok((device, desc_len)) => {
                        if has_report(&report_descriptor[..desc_len], REPORT_ID) {
                            if let Some(product) =
                                Product::from_usb_product(device_info.product_id())
                            {
                                Compatibility::Tested(device, product)
                            } else {
                                Compatibility::Untested(device)
                            }
                        } else {
                            // Incompatible due to missing required report.
                            Compatibility::Incompatible(device_info)
                        }
                    }

                    Err(error) => {
                        eprintln!("USB HID device error: {}", error);

                        // Incompatible due to error.
                        Compatibility::Incompatible(device_info)
                    }
                }
            } else {
                // Incompatible due to incorrect vendor.
                Compatibility::Incompatible(device_info)
            }
        })
        .collect()
}

/// Returns the first compatible device, preferring devices tested to work.
pub fn first_compatible_device(api: &HidApi) -> Option<Compatibility> {
    let mut untested = None;
    for compat in device_compatibility(api) {
        match compat {
            Compatibility::Tested(_, _) => return Some(compat),
            Compatibility::Untested(_) => {
                if untested.is_none() {
                    untested = Some(compat)
                }
            }
            Compatibility::Incompatible(_) => {}
        }
    }

    untested
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
