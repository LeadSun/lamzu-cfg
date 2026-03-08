use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("USB HID API error")]
    Hid(#[from] HidError),

    #[error("Mouse profile data is invalid")]
    InvalidProfile(String),

    #[error("Mouse returned an error code")]
    MouseErrorResponse(u8),

    #[error("No compatible devices found")]
    NoDevice,

    #[error("Untested device found")]
    UntestedDevice,

    #[error("No valid response for request")]
    NoResponse,
}
