mod profile_rw;
mod raw_data;
mod raw_profile;
mod report;

use crate::device::{checksum, Mouse};
use crate::profile::Profile;
use hidapi::HidDevice;
use raw_profile::RawProfile;
use report::{make_request, StandardReport};

// Checksum algorithms used.
type Sum171 = checksum::SumComplement8<171>;
type Sum181 = checksum::SumComplement8<181>;

const NUM_BUTTONS: u8 = 6;
const NUM_PROFILES: usize = 4;

/// Lamzu Atlantis mouse interface.
pub struct Atlantis;

impl Mouse for Atlantis {
    fn profile(&self, device: &HidDevice, index: usize) -> crate::Result<Profile> {
        // Only the active profile can be accessed, so store the current profile and
        // switch.
        let active_profile = self.active_profile_index(device)?;
        if active_profile != index {
            self.set_active_profile_index(device, index)?;
        }

        let profile = RawProfile::read_from_mouse(device, NUM_BUTTONS)?.try_into();

        // Switch back to original profile.
        if active_profile != index {
            self.set_active_profile_index(device, active_profile)?;
        }

        profile
    }

    fn set_profile(
        &self,
        device: &HidDevice,
        index: usize,
        profile: &Profile,
    ) -> crate::Result<()> {
        // Only the active profile can be accessed, so store the current profile and
        // switch.
        let active_profile = self.active_profile_index(device)?;
        if active_profile != index {
            self.set_active_profile_index(device, index)?;
        }

        RawProfile::try_from(profile)?.write_to_mouse(device, NUM_BUTTONS)?;

        // Switch back to original profile.
        if active_profile != index {
            self.set_active_profile_index(device, active_profile)?;
        }

        Ok(())
    }

    fn profiles(&self, device: &HidDevice) -> crate::Result<Vec<Profile>> {
        let active_profile = self.active_profile_index(device)?;
        let profiles = (0..NUM_PROFILES)
            .into_iter()
            .map(|i| {
                self.set_active_profile_index(device, i)?;
                RawProfile::read_from_mouse(device, 6)?.try_into()
            })
            .collect();
        self.set_active_profile_index(device, active_profile)?;

        profiles
    }

    fn set_profiles(&self, device: &HidDevice, profiles: &[Profile]) -> crate::Result<()> {
        let active_profile = self.active_profile_index(device)?;
        for (i, raw_profile) in profiles
            .iter()
            .map(|profile| RawProfile::try_from(profile))
            .enumerate()
        {
            self.set_active_profile_index(device, i)?;
            raw_profile?.write_to_mouse(device, NUM_BUTTONS)?;
        }
        self.set_active_profile_index(device, active_profile)?;

        Ok(())
    }

    fn active_profile_index(&self, device: &HidDevice) -> crate::Result<usize> {
        Ok(make_request(device, &StandardReport::read_active_profile())?.into_data()?[0] as usize)
    }

    fn set_active_profile_index(&self, device: &HidDevice, index: usize) -> crate::Result<()> {
        make_request(device, &StandardReport::write_active_profile(index as u8))?.data()?;

        Ok(())
    }
}
