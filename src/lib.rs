mod data;
mod device;
pub use device::first_compatible_device;
mod error;
pub use error::Error;
mod profile;
pub use profile::Profile;

pub type Result<T> = std::result::Result<T, error::Error>;
