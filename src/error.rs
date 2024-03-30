use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No compatible devices found")]
    NoDevice,

    #[error("USB HID API error")]
    Hid(#[from] HidError),
}
