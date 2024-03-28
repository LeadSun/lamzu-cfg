use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No compatible devices found")]
    NoDevice,

    #[error("Failed to connect to HID device")]
    HidConnect(#[from] HidError),
}
