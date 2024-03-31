use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("BinRw error")]
    BinRw(#[from] binrw::Error),

    #[error("USB HID API error")]
    Hid(#[from] HidError),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Mouse returned an error code")]
    MouseErrorResponse(u8),

    #[error("No compatible devices found")]
    NoDevice,

    #[error("No valid response for request")]
    NoResponse,

    #[error("Received a different report than expected")]
    UnexpectedReport,
}
