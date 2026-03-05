mod atlantis;
pub use atlantis::Atlantis;
mod device;
mod error;
pub use error::Error;
pub mod profile;
pub use profile::Profile;

pub type Result<T> = std::result::Result<T, error::Error>;

/// Trait for supported mice that can be configured via profiles.
pub trait Mouse {
    const NUM_PROFILES: usize;

    /// Returns a specific profile from the device.
    fn profile(&self, index: usize) -> crate::Result<Profile>;

    /// Write to a specific profile on the device.
    fn set_profile(&self, index: usize, profile: &Profile) -> crate::Result<()>;

    /// Returns all profiles from the device.
    fn profiles(&self) -> crate::Result<Vec<Profile>> {
        (0..Self::NUM_PROFILES)
            .into_iter()
            .map(|i| self.profile(i))
            .collect()
    }

    /// Write multiple profiles to the device.
    fn set_profiles(&self, profiles: &[Profile]) -> crate::Result<()> {
        for (i, profile) in profiles.iter().enumerate() {
            self.set_profile(i, profile)?;
        }
        Ok(())
    }

    /// Returns the index of the currently active profile.
    fn active_profile(&self) -> crate::Result<usize>;

    /// Set the active profile by index.
    fn set_active_profile(&self, index: usize) -> crate::Result<()>;

    /// Returns the battery voltage in millivolts.
    fn battery_voltage(&self) -> crate::Result<u16>;

    /// Returns the rough battery charge percentage.
    fn battery_percentage(&self) -> crate::Result<u8>;
}
