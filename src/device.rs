use hidapi::{HidApi, HidDevice};

const VENDOR_ID: u16 = 0x3554;
const SUPPORTED_PRODUCTS: [u16; 2] = [0xf50d, 0xf50f];

pub fn first_compatible_device(api: &HidApi) -> crate::Result<HidDevice> {
    match api.device_list().find(|info| {
        info.vendor_id() == VENDOR_ID && SUPPORTED_PRODUCTS.contains(&info.product_id())
    }) {
        Some(device_info) => Ok(device_info.open_device(&api)?),
        None => Err(crate::Error::NoDevice),
    }
}
