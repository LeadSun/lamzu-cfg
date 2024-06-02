use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("BinRw error")]
    BinRw(#[from] binrw::Error),

    #[error("USB HID API error")]
    Hid(#[from] HidError),

    #[error("Mouse profile data conversion error")]
    InvalidConversion(String),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization / deserialization error")]
    Json(#[from] serde_json::Error),

    #[error("Mouse returned an error code")]
    MouseErrorResponse(u8),

    #[error("No compatible devices found")]
    NoDevice,

    #[error("No valid response for request")]
    NoResponse,

    #[error("RON serialization / deserialization error")]
    RonError(#[from] ron::Error),

    #[error("Received a different report than expected")]
    UnexpectedReport,
}
