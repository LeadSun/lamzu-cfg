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

    #[error("Device is not compatible")]
    Incompatible,

    #[error("No valid response for request")]
    NoResponse,
}
