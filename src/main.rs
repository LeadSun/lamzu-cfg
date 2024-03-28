use hidapi::HidApi;
use lamzu_cfg::first_compatible_device;

fn main() {
    let api = HidApi::new().unwrap();
    let device = first_compatible_device(&api).unwrap();

    println!(
        "Found device: {} {}",
        device
            .get_manufacturer_string()
            .unwrap()
            .unwrap_or("Unknown".to_string()),
        device
            .get_product_string()
            .unwrap()
            .unwrap_or("Unknown".to_string())
    );
}
