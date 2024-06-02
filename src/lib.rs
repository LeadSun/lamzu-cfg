mod data;
pub mod device;
mod error;
pub use error::Error;
mod profile;
pub use profile::Profile;

pub type Result<T> = std::result::Result<T, error::Error>;
