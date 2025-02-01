use crate::device::atlantis::profile_rw::{ProfileReader, ProfileWriter};
use crate::device::atlantis::{raw_data::*, Sum171, Sum181};
use crate::device::{checksum, BinRw};
use crate::{data::*, Profile};
use binrw::{binrw, BinRead, BinWrite};
use hidapi::HidDevice;
use std::collections::HashMap;
use std::fmt;
use std::io::SeekFrom;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Range};

/// Lamzu-Atlantis-specific profile data that can be read / written to mouse.
///
/// All settings are optional to allow for partial profile writes. Profile reads
/// should always result in `Some` values.
#[binrw]
#[brw(import { num_buttons: u8 })]
#[br(pre_assert(num_buttons <= 16))]
#[derive(Debug, Default)]
pub struct RawProfile {
    #[brw(args { length: 2 })]
    poll_rate: Setting<u8, Sum171>,

    #[brw(args { length: 2 }, assert((1..=8).contains(&dpi_count.unwrap_or(1))))]
    dpi_count: Setting<u8, Sum171>,

    #[brw(args { length: 2 }, assert(current_dpi_index.unwrap_or(0) < 8))]
    current_dpi_index: Setting<u8, Sum171>,

    #[brw(args { length: 2 }, seek_before = SeekFrom::Current(4))]
    lift_off_distance: Setting<u8, Sum171>,

    #[brw(seek_before = SeekFrom::Start(12))]
    #[br(args { count: dpi_count.unwrap() as usize, inner: binrw::args! { length: 4 } })]
    #[bw(args { length: 4 }, assert(dpis.len() <= 8))]
    dpis: Vec<Setting<RawDpi, Sum171>>,

    #[brw(seek_before = SeekFrom::Start(44))]
    #[br(args { count: dpi_count.unwrap() as usize, inner: binrw::args! { length: 4 } })]
    #[bw(args { length: 4 }, assert(dpi_colors.len() <= 8))]
    dpi_colors: Vec<Setting<RawColor, Sum171>>,

    #[brw(seek_before = SeekFrom::Start(96))]
    #[br(args { count: num_buttons as usize, inner: binrw::args! { length: 4 } })]
    #[bw(args { length: 4 }, assert(button_actions.len() <= num_buttons as usize))]
    button_actions: Vec<Setting<PaddedRawAction, Sum171>>,

    #[brw(args { length: 2 }, seek_before = SeekFrom::Start(169))]
    debounce_ms: Setting<u8, Sum171>,

    #[brw(args { length: 2 })]
    motion_sync: Setting<u8, Sum171>,

    #[brw(args { length: 2 }, seek_before = SeekFrom::Current(2))]
    angle_snapping: Setting<u8, Sum171>,

    #[brw(args { length: 2 })]
    ripple_control: Setting<u8, Sum171>,

    #[brw(args { length: 2 }, seek_before = SeekFrom::Current(2))]
    peak_performance: Setting<u8, Sum171>,

    #[brw(args { length: 2 })]
    peak_performance_time: Setting<u8, Sum171>,

    #[brw(args { length: 2 })]
    high_performance: Setting<u8, Sum171>,

    #[br(ignore)] // Read separately.
    #[bw(args { length: 32 }, seek_before = SeekFrom::Start(0x0100))]
    #[bw(assert(combos.len() <= num_buttons as usize))]
    combos: Vec<Setting<RawCombo, Sum171>>,

    #[br(ignore)] // Read separately.
    #[bw(args { length: 384 }, seek_before = SeekFrom::Start(0x0300))]
    #[bw(assert(macros.len() <= num_buttons as usize))]
    macros: Vec<Setting<RawMacro, Sum181>>,
}

impl RawProfile {
    pub fn read_from_mouse(device: &HidDevice, num_buttons: u8) -> crate::Result<Self> {
        let mut profile = Self::read_be_args(
            &mut ProfileReader::new(device, 0),
            binrw::args! { num_buttons },
        )?;

        // Manually read combos and macros so checksum errors from uninitialized slots
        // can be handled.
        for i in 0..(num_buttons as usize) {
            profile.combos.push(Setting::new(
                RawCombo::read_be(&mut ProfileReader::new(device, 0x0100 + (i * 32))).ok(),
            ));

            profile.macros.push(Setting::new(
                RawMacro::read_be(&mut ProfileReader::new(device, 0x0300 + (i * 384))).ok(),
            ));
        }

        Ok(profile)
    }

    pub fn write_to_mouse(&self, device: &HidDevice, num_buttons: u8) -> crate::Result<()> {
        self.write_be_args(
            &mut ProfileWriter::new(device, 0),
            binrw::args! { num_buttons },
        )?;

        Ok(())
    }
}

impl TryFrom<&Profile> for RawProfile {
    type Error = crate::Error;

    fn try_from(profile: &Profile) -> crate::Result<Self> {
        assert_range(0..9, profile.dpis.len())?;
        assert_range(0..9, profile.dpi_colors.len())?;
        assert_range(0..17, profile.button_actions.len())?;
        assert_range_opt(0..8, &profile.current_dpi_index)?;
        assert_range_opt(0..2, &profile.lift_off_distance)?;
        assert_range_opt(0..16, &profile.debounce_ms)?;
        assert_range_opt(0..2551, &profile.peak_performance_time)?;

        let (button_actions, combos, macros) = profile_to_raw_actions_combos_macros(profile)?;

        Ok(Self {
            poll_rate: Setting::new(match profile.poll_rate {
                Some(1000) => Some(1),
                Some(500) => Some(2),
                Some(250) => Some(4),
                Some(125) => Some(8),
                Some(poll_rate) => {
                    return Err(crate::Error::InvalidConversion(format!(
                        "Invalid poll rate value to write to mouse: {}",
                        poll_rate
                    )))
                }
                None => None,
            }),
            dpi_count: Setting::new(
                Some((profile.dpis.len() as u8).max(profile.dpi_colors.len() as u8))
                    .filter(|len| *len > 0), // Don't write if dpis / colors are empty.
            ),
            current_dpi_index: Setting::new(profile.current_dpi_index.map(|dpi| dpi as u8)),
            lift_off_distance: Setting::new(profile.lift_off_distance),
            dpis: profile
                .dpis
                .iter()
                .cloned()
                .map(RawDpi::from)
                .map(Setting::from)
                .collect(),
            dpi_colors: profile
                .dpi_colors
                .iter()
                .cloned()
                .map(RawColor::from)
                .map(Setting::from)
                .collect(),
            button_actions: button_actions
                .into_iter()
                .map(PaddedRawAction::from)
                .map(Setting::from)
                .collect(),
            debounce_ms: Setting::new(profile.debounce_ms),
            motion_sync: Setting::new(profile.motion_sync.map(u8::from)),
            angle_snapping: Setting::new(profile.angle_snapping.map(u8::from)),
            ripple_control: Setting::new(profile.ripple_control.map(u8::from)),
            peak_performance: Setting::new(profile.peak_performance.map(u8::from)),
            peak_performance_time: Setting::new(
                profile.peak_performance_time.map(|ppt| (ppt / 10) as u8),
            ),
            high_performance: Setting::new(profile.high_performance.map(u8::from)),
            combos: combos.into_iter().map(Setting::new).collect(),
            macros: macros.into_iter().map(Setting::new).collect(),
        })
    }
}

impl TryFrom<RawProfile> for Profile {
    type Error = crate::Error;

    fn try_from(raw_profile: RawProfile) -> crate::Result<Self> {
        let (button_actions, macros) = raw_profile_to_actions_macros(&raw_profile)?;

        Ok(Self {
            poll_rate: match raw_profile.poll_rate.inner {
                Some(1) => Some(1000),
                Some(2) => Some(500),
                Some(4) => Some(250),
                Some(8) => Some(125),
                Some(poll_rate) => {
                    return Err(crate::Error::InvalidConversion(format!(
                        "Invalid raw poll rate value from mouse: {}",
                        poll_rate
                    )))
                }
                None => None,
            },
            current_dpi_index: raw_profile.current_dpi_index.map(usize::from),
            lift_off_distance: raw_profile.lift_off_distance.inner,
            dpis: raw_profile
                .dpis
                .iter()
                .map(|dpi| Dpi::from(dpi.expect("Unreachable")))
                .collect(),
            dpi_colors: raw_profile
                .dpi_colors
                .iter()
                .map(|color| Color::from(color.expect("Unreachable")))
                .collect(),
            debounce_ms: raw_profile.debounce_ms.inner,
            motion_sync: raw_profile.motion_sync.inner.map(to_bool),
            angle_snapping: raw_profile.angle_snapping.inner.map(to_bool),
            ripple_control: raw_profile.ripple_control.inner.map(to_bool),
            peak_performance: raw_profile.peak_performance.inner.map(to_bool),
            peak_performance_time: raw_profile
                .peak_performance_time
                .inner
                .map(|ppt| ppt as u16 * 10),
            high_performance: raw_profile.high_performance.inner.map(to_bool),
            button_actions,
            macros,
        })
    }
}

/// Wraps a setting value to make it skippable and appended with a checksum.
#[binrw]
#[brw(import { length: u16 })] // Length in bytes of whole setting including checksum.
#[derive(Clone)]
struct Setting<T: BinRw + Clone, A: checksum::Algorithm8 + Default> {
    #[brw(restore_position)]
    #[br(map = |checksummed: checksum::Append8<T, A>| Some(checksummed.into_inner()))]
    #[bw(map = |inner| inner.as_ref().map(|val| checksum::Append8::<T, A>::new((*val).clone())))]
    inner: Option<T>,

    // binrw pad_* attributes write 0 bytes so seek instead to avoid overwrite.
    #[brw(seek_before = SeekFrom::Current(length as i64))]
    phantom: PhantomData<A>,
}

impl<T: BinRw + Clone, A: checksum::Algorithm8 + Default> Setting<T, A> {
    fn new(inner: Option<T>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }
}

impl<T: BinRw + Clone, A: checksum::Algorithm8 + Default> Default for Setting<T, A> {
    fn default() -> Self {
        Self {
            inner: None,
            phantom: PhantomData,
        }
    }
}

impl<T: BinRw + Clone, A: checksum::Algorithm8 + Default> Deref for Setting<T, A> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: BinRw + Clone, A: checksum::Algorithm8 + Default> DerefMut for Setting<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: BinRw + Clone + fmt::Debug, A: checksum::Algorithm8 + Default> fmt::Debug
    for Setting<T, A>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: BinRw + Clone, A: checksum::Algorithm8 + Default> From<T> for Setting<T, A> {
    fn from(inner: T) -> Self {
        Self {
            inner: Some(inner),
            phantom: PhantomData,
        }
    }
}

/// Converts actions, combos, and macros from a profile to their raw versions.
fn profile_to_raw_actions_combos_macros(
    profile: &Profile,
) -> crate::Result<(Vec<RawAction>, Vec<Option<RawCombo>>, Vec<Option<RawMacro>>)> {
    // Each combo slot is tied to the button action of the same index.
    let mut combos: Vec<Option<RawCombo>> = vec![None; profile.button_actions.len()];

    // Macro slots aren't tied to a specific button.
    let mut macros: Vec<Option<RawMacro>> = Vec::new();

    let button_actions: Vec<RawAction> = profile
        .button_actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            Ok(match action {
                Action::Disabled => RawAction::Disabled,

                Action::LeftClick => RawAction::Button { id: 1 },
                Action::RightClick => RawAction::Button { id: 2 },
                Action::MiddleClick => RawAction::Button { id: 4 },
                Action::BackClick => RawAction::Button { id: 8 },
                Action::ForwardClick => RawAction::Button { id: 16 },

                Action::DpiLoop => RawAction::DpiLoop,
                Action::DpiUp => RawAction::DpiUp,
                Action::DpiDown => RawAction::DpiDown,
                Action::DpiLock { dpi } => RawAction::DpiLock {
                    dpi: dpi_to_raw(*dpi),
                },

                Action::PollRateLoop => RawAction::PollRateLoop,

                Action::WheelLeft => RawAction::WheelLeft,
                Action::WheelRight => RawAction::WheelRight,
                Action::WheelUp => RawAction::WheelUp,
                Action::WheelDown => RawAction::WheelDown,

                Action::Fire { interval, repeat } => RawAction::Fire {
                    interval: *interval,
                    repeat: *repeat,
                },

                Action::Combo { events } => {
                    combos[i] = RawCombo::from(events.clone()).into();
                    RawAction::Combo
                }

                Action::Macro { name } => {
                    let events =
                        profile
                            .macros
                            .get(name)
                            .ok_or(crate::Error::InvalidConversion(format!(
                                "Macro does not exist: '{}'",
                                name
                            )))?;
                    macros.push(Some(RawMacro::new(
                        name.clone(),
                        events.iter().cloned().map(RawMacroEvent::from).collect(),
                    )));
                    RawAction::Macro {
                        index: macros.len() as u8 - 1,
                    }
                }
            })
        })
        .collect::<crate::Result<Vec<_>>>()?;

    Ok((button_actions, combos, macros))
}

/// Converts actions and macros from a raw profile to their standard versions.
fn raw_profile_to_actions_macros(
    raw_profile: &RawProfile,
) -> crate::Result<(Vec<Action>, HashMap<String, Vec<MacroEvent>>)> {
    let mut macros: HashMap<String, Vec<MacroEvent>> = HashMap::new();

    let actions: Vec<Action> = raw_profile
        .button_actions
        .iter()
        .enumerate()
        .map(|(i, raw_action)| {
            Ok(match raw_action.inner.unwrap().action {
                RawAction::Disabled => Action::Disabled,

                RawAction::Button { id } => match id {
                    1 => Action::LeftClick,
                    2 => Action::RightClick,
                    4 => Action::MiddleClick,
                    8 => Action::BackClick,
                    16 => Action::ForwardClick,
                    _ => {
                        return Err(crate::Error::InvalidConversion(format!(
                            "Invalid button ID from raw: {}",
                            id
                        )))
                    }
                },

                RawAction::DpiLoop => Action::DpiLoop,
                RawAction::DpiUp => Action::DpiUp,
                RawAction::DpiDown => Action::DpiDown,

                RawAction::WheelLeft => Action::WheelLeft,
                RawAction::WheelRight => Action::WheelRight,

                RawAction::Fire { interval, repeat } => Action::Fire { interval, repeat },

                RawAction::Combo => Action::Combo {
                    events: raw_profile.combos[i]
                        .inner
                        .clone()
                        .ok_or(crate::Error::InvalidConversion(format!(
                            "Raw combo does not exist: {}",
                            i
                        )))?
                        .try_into()?,
                },
                RawAction::Macro { index } => {
                    let raw_macro = raw_profile
                        .macros
                        .get(index as usize)
                        .map(|setting| setting.inner.as_ref())
                        .flatten()
                        .ok_or(crate::Error::InvalidConversion(format!(
                            "Raw macro does not exist: {}",
                            i
                        )))?;
                    macros.insert(
                        raw_macro.name.clone(),
                        raw_macro
                            .events
                            .iter()
                            .cloned()
                            .map(MacroEvent::try_from)
                            .collect::<crate::Result<Vec<_>>>()?,
                    );

                    Action::Macro {
                        name: String::new(),
                    }
                }

                RawAction::PollRateLoop => Action::PollRateLoop,
                RawAction::DpiLock { dpi } => Action::DpiLock {
                    dpi: dpi_from_raw(dpi),
                },

                RawAction::WheelUp => Action::WheelUp,
                RawAction::WheelDown => Action::WheelDown,
            })
        })
        .collect::<crate::Result<Vec<_>>>()?;

    Ok((actions, macros))
}

fn to_bool(byte: u8) -> bool {
    byte != 0
}

/// Returns an error if `range` is `Some` and does not contain `val`.
fn assert_range_opt<T: PartialOrd + PartialEq + std::fmt::Display>(
    range: Range<T>,
    val: &Option<T>,
) -> crate::Result<()> {
    match val {
        Some(v) => {
            if range.contains(v) {
                Ok(())
            } else {
                Err(crate::Error::InvalidConversion(format!(
                    "Value '{}' out of range '{}..{}'",
                    v, range.start, range.end
                )))
            }
        }

        None => Ok(()),
    }
}

/// Returns an error if `range` does not contain `val`.
fn assert_range<T: PartialOrd + PartialEq + std::fmt::Display>(
    range: Range<T>,
    val: T,
) -> crate::Result<()> {
    assert_range_opt(range, &Some(val))
}
