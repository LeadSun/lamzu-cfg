mod hid;
use hid::*;

use crate::device::{device_compatibility, Compatibility, Product};
use crate::profile::{
    Action, Button, Color, KeyEvent, Macro, MacroEvent, MacroMode, Profile, Resolution,
};
use crate::Mouse;
use hidapi::{HidApi, HidDevice};
use keycode::{KeyMap, KeyMappingId, KeyModifiers, KeyState};
use std::collections::HashMap;
use std::ops::{RangeBounds, RangeInclusive};

const BATTERY_MIN_MILLIVOLTS: u16 = 3050;
const BATTERY_MAX_MILLIVOLTS: u16 = 4200;

mod address {
    pub const POLL_RATE: usize = 0;
    pub const RESOLUTION_COUNT: usize = 2;
    pub const RESOLUTION_INDEX: usize = 4;
    pub const LIFT_OFF_DISTANCE: usize = 10;
    pub const RESOLUTIONS: usize = 12;
    pub const RESOLUTION_COLORS: usize = 44;
    pub const CHARGING_LED: usize = 84;
    pub const BUTTON_ACTIONS: usize = 96;
    pub const DEBOUNCE_MS: usize = 169;
    pub const MOTION_SYNC: usize = 171;
    pub const ANGLE_SNAPPING: usize = 175;
    pub const RIPPLE_CONTROL: usize = 177;
    pub const PEAK_PERFORMANCE: usize = 181;
    pub const PEAK_PERFORMANCE_TIME: usize = 183;
    pub const HIGH_PERFORMANCE: usize = 185;
    pub const COMBOS: usize = 256;
    pub const MACROS: usize = 768;
}

const POLL_RATE_MAP: [(u16, u8); 7] = [
    (1000, 1),
    (500, 2),
    (250, 4),
    (125, 8),
    (2000, 16),
    (4000, 32),
    (8000, 64),
];

const NUM_BUTTONS: usize = 6;
const BUTTONS: [Button; NUM_BUTTONS] = [
    Button::Left,
    Button::Right,
    Button::Middle,
    Button::Back,
    Button::Forward,
    Button::Bottom,
];

const MAX_RESOLUTION_COUNT: usize = 8;
const MAX_RESOLUTION: u16 = 26000;
const RANGE_LIFT_OFF_DISTANCE: RangeInclusive<usize> = 1..=2;
const MAX_DEBOUNCE_MS: usize = 15;
const MAX_COMBO_EVENTS: usize = 6;
const MAX_MACRO_NAME_LEN: usize = 30;
const MAX_MACRO_EVENTS: usize = 70;

pub struct Atlantis {
    device: HidDevice,
    product: Product,
}

impl Atlantis {
    /// Connect to the first compatible Atlantis mouse, optionally accepting
    /// untested mice.
    pub fn connect(force: bool) -> crate::Result<Self> {
        let api = HidApi::new()?;
        let device_compat = device_compatibility(&api)
            .into_iter()
            .reduce(|acc, compat| match acc {
                Compatibility::Tested(_, _) => acc,
                Compatibility::Untested(_) => match compat {
                    Compatibility::Tested(_, _) => compat,
                    _ => acc,
                },
                Compatibility::Incompatible(_) => compat,
            })
            .ok_or(crate::Error::NoDevice)?;

        match device_compat {
            Compatibility::Tested(device, product) => Ok(Self::new(device, product)),
            Compatibility::Untested(device) => {
                if force {
                    Ok(Self::new(device, Product::default()))
                } else {
                    Err(crate::Error::UntestedDevice)
                }
            }
            Compatibility::Incompatible(_) => Err(crate::Error::NoDevice),
        }
    }

    fn new(device: HidDevice, product: Product) -> Self {
        Self { device, product }
    }

    fn read_flash_checked(&self, address: usize, length: usize) -> crate::Result<Vec<u8>> {
        let mut data = read_flash(&self.device, address, length + 1)?;
        if checksum(&data) == 0 {
            data.pop();
            Ok(data)
        } else {
            Err(crate::Error::InvalidConversion(format!(
                "Checksum mismatch at {address:40X}: {data:?}"
            )))
        }
    }

    fn write_flash_checked(&self, address: usize, data: &[u8]) -> crate::Result<()> {
        let mut data = data.to_vec();
        data.push(checksum(&data));
        write_flash(&self.device, address, data)
    }

    fn read_byte(&self, address: usize) -> crate::Result<u8> {
        Ok(self.read_flash_checked(address, 1)?[0])
    }

    fn write_byte(&self, address: usize, val: u8) -> crate::Result<()> {
        self.write_flash_checked(address, &[val])
    }

    fn read_bool(&self, address: usize) -> crate::Result<bool> {
        Ok(self.read_byte(address)? != 0)
    }

    fn write_bool(&self, address: usize, val: bool) -> crate::Result<()> {
        self.write_byte(address, val as u8)
    }

    fn poll_rate(&self) -> crate::Result<u16> {
        let raw = self.read_byte(address::POLL_RATE)?;
        Ok(POLL_RATE_MAP
            .iter()
            .find(|(_, r)| *r == raw)
            .ok_or(crate::Error::InvalidConversion(format!(
                "Invalid poll rate value '{}' from mouse",
                raw
            )))?
            .0)
    }

    fn set_poll_rate(&self, mut poll_rate: u16) -> crate::Result<()> {
        if poll_rate > self.product.max_poll_rate() {
            eprintln!(
                "Warning: Desired poll rate is unsupported by mouse. Reducing to {}Hz.",
                self.product.max_poll_rate()
            );
            poll_rate = self.product.max_poll_rate();
        }
        let raw = POLL_RATE_MAP
            .iter()
            .find(|(p, _)| *p == poll_rate)
            .ok_or(crate::Error::InvalidConversion(format!(
                "Poll rate {poll_rate} is not supported",
            )))?
            .1;
        self.write_byte(address::POLL_RATE, raw)
    }

    fn resolution_index(&self) -> crate::Result<u8> {
        let raw = self.read_byte(address::RESOLUTION_INDEX)?;
        assert_range(0..MAX_RESOLUTION_COUNT, raw)?;
        Ok(raw)
    }

    fn set_resolution_index(&self, resolution_index: u8) -> crate::Result<()> {
        assert_range(0..MAX_RESOLUTION_COUNT, resolution_index)?;
        self.write_byte(address::RESOLUTION_INDEX, resolution_index)
    }

    fn resolution_count(&self) -> crate::Result<u8> {
        let raw = self.read_byte(address::RESOLUTION_COUNT)?;
        assert_range(1..=MAX_RESOLUTION_COUNT, raw)?;
        Ok(raw)
    }

    fn set_resolution_count(&self, count: u8) -> crate::Result<()> {
        assert_range(1..=MAX_RESOLUTION_COUNT, count)?;
        self.write_byte(address::RESOLUTION_COUNT, count)
    }

    fn resolution(&self, index: usize) -> crate::Result<Resolution> {
        assert_range(0..MAX_RESOLUTION_COUNT, index)?;
        let resolution_raw = self.read_flash_checked(address::RESOLUTIONS + (index * 4), 3)?;
        let resolution = Resolution::new(
            resolution_from_raw(resolution_raw[0]),
            resolution_from_raw(resolution_raw[1]),
        );
        assert_range(50..=MAX_RESOLUTION, resolution.x)?;
        assert_range(50..=MAX_RESOLUTION, resolution.y)?;
        Ok(resolution)
    }

    fn set_resolution(&self, index: usize, resolution: &Resolution) -> crate::Result<()> {
        assert_range(0..MAX_RESOLUTION_COUNT, index)?;
        assert_range(50..=MAX_RESOLUTION, resolution.x)?;
        assert_range(50..=MAX_RESOLUTION, resolution.y)?;
        self.write_flash_checked(
            address::RESOLUTIONS + (index * 4),
            &[
                resolution_to_raw(resolution.x),
                resolution_to_raw(resolution.y),
                0,
            ],
        )
    }

    fn resolutions(&self) -> crate::Result<Vec<Resolution>> {
        let count = self.resolution_count()? as usize;
        let mut resolutions = Vec::new();
        for i in 0..count {
            resolutions.push(self.resolution(i)?);
        }
        Ok(resolutions)
    }

    fn set_resolutions(&self, resolutions: &[Resolution]) -> crate::Result<()> {
        self.set_resolution_count(resolutions.len() as u8)?;
        for (i, resolution) in resolutions.iter().enumerate() {
            self.set_resolution(i, resolution)?;
        }
        Ok(())
    }

    fn resolution_color(&self, index: usize) -> crate::Result<Color> {
        assert_range(0..MAX_RESOLUTION_COUNT, index)?;
        let raw = self.read_flash_checked(address::RESOLUTION_COLORS + (index * 4), 3)?;
        Ok(Color::new(raw[0], raw[1], raw[2]))
    }

    fn set_resolution_color(&self, index: usize, color: &Color) -> crate::Result<()> {
        assert_range(0..MAX_RESOLUTION_COUNT, index)?;
        self.write_flash_checked(
            address::RESOLUTION_COLORS + (index * 4),
            &[color.red, color.green, color.blue],
        )
    }

    fn resolution_colors(&self) -> crate::Result<Vec<Color>> {
        let count = self.resolution_count()? as usize;
        let mut colors = Vec::new();
        for i in 0..count {
            colors.push(self.resolution_color(i)?);
        }
        Ok(colors)
    }

    fn set_resolution_colors(&self, colors: &[Color]) -> crate::Result<()> {
        assert_range(1..=MAX_RESOLUTION_COUNT, colors.len())?;
        for (i, color) in colors.iter().enumerate() {
            self.set_resolution_color(i, color)?;
        }
        Ok(())
    }

    fn charging_color(&self) -> crate::Result<Color> {
        let raw = self.read_flash_checked(address::CHARGING_LED, 3)?;
        Ok(Color::new(raw[0], raw[1], raw[2]))
    }

    fn set_charging_color(&self, color: &Color) -> crate::Result<()> {
        self.write_flash_checked(address::CHARGING_LED, &[color.red, color.green, color.blue])
    }

    fn lift_off_distance(&self) -> crate::Result<u8> {
        let raw = self.read_byte(address::LIFT_OFF_DISTANCE)?;
        assert_range(RANGE_LIFT_OFF_DISTANCE, raw)?;
        Ok(raw)
    }

    fn set_lift_off_distance(&self, lod: u8) -> crate::Result<()> {
        assert_range(RANGE_LIFT_OFF_DISTANCE, lod)?;
        self.write_byte(address::LIFT_OFF_DISTANCE, lod)?;
        Ok(())
    }

    fn debounce_ms(&self) -> crate::Result<u8> {
        let debounce_ms = self.read_byte(address::DEBOUNCE_MS)?;
        assert_range(0..=MAX_DEBOUNCE_MS, debounce_ms)?;
        Ok(debounce_ms)
    }

    fn set_debounce_ms(&self, debounce_ms: u8) -> crate::Result<()> {
        assert_range(0..=MAX_DEBOUNCE_MS, debounce_ms)?;
        self.write_byte(address::DEBOUNCE_MS, debounce_ms)
    }

    fn motion_sync(&self) -> crate::Result<bool> {
        self.read_bool(address::MOTION_SYNC)
    }

    fn set_motion_sync(&self, motion_sync: bool) -> crate::Result<()> {
        self.write_bool(address::MOTION_SYNC, motion_sync)
    }

    fn angle_snapping(&self) -> crate::Result<bool> {
        self.read_bool(address::ANGLE_SNAPPING)
    }

    fn set_angle_snapping(&self, angle_snapping: bool) -> crate::Result<()> {
        self.write_bool(address::ANGLE_SNAPPING, angle_snapping)
    }

    fn ripple_control(&self) -> crate::Result<bool> {
        self.read_bool(address::RIPPLE_CONTROL)
    }

    fn set_ripple_control(&self, ripple_control: bool) -> crate::Result<()> {
        self.write_bool(address::RIPPLE_CONTROL, ripple_control)
    }

    fn peak_performance(&self) -> crate::Result<bool> {
        self.read_bool(address::PEAK_PERFORMANCE)
    }

    fn set_peak_performance(&self, peak_performance: bool) -> crate::Result<()> {
        self.write_bool(address::PEAK_PERFORMANCE, peak_performance)
    }

    fn peak_performance_seconds(&self) -> crate::Result<u16> {
        let raw = self.read_byte(address::PEAK_PERFORMANCE_TIME)?;
        Ok(raw as u16 * 10)
    }

    fn set_peak_performance_seconds(&self, peak_performance_seconds: u16) -> crate::Result<()> {
        let raw = (peak_performance_seconds / 10).min(u8::MAX as u16) as u8;
        self.write_byte(address::PEAK_PERFORMANCE_TIME, raw)
    }

    fn high_performance(&self) -> crate::Result<bool> {
        self.read_bool(address::HIGH_PERFORMANCE)
    }

    fn set_high_performance(&self, high_performance: bool) -> crate::Result<()> {
        self.write_bool(address::HIGH_PERFORMANCE, high_performance)
    }

    fn button_mappings(&self) -> crate::Result<(HashMap<Button, Action>, HashMap<String, Macro>)> {
        let mut button_map = HashMap::new();
        let mut macros = HashMap::new();
        for (i, button) in BUTTONS.iter().enumerate() {
            let action_raw = self.read_flash_checked(address::BUTTON_ACTIONS + (i * 4), 3)?;
            match &action_raw[..] {
                [5, 0, 0] => {
                    button_map.insert(
                        *button,
                        Action::Combo {
                            events: self.key_combo(i)?,
                        },
                    );
                }

                [6, macro_index, repeat] => {
                    if *macro_index as usize != i {
                        eprintln!("Macro index ({macro_index}) doesn't match button ({i}). Corrupted action?");
                    }
                    let mode = match repeat {
                        255 => MacroMode::UntilPress,
                        254 => MacroMode::Hold,
                        253 => MacroMode::Toggle,
                        x => MacroMode::Repeat(*x),
                    };
                    let (name, events) = self.get_macro(i)?;
                    macros.insert(name.clone(), Macro { mode, events });
                    button_map.insert(*button, Action::Macro { name });
                }

                _ => {
                    button_map.insert(*button, action_from_raw(&action_raw)?);
                }
            }
        }
        Ok((button_map, macros))
    }

    fn set_button_mappings(
        &self,
        button_map: &HashMap<Button, Action>,
        macros: &HashMap<String, Macro>,
    ) -> crate::Result<()> {
        for (i, button) in BUTTONS.iter().enumerate() {
            if let Some(action) = button_map.get(button) {
                match action {
                    Action::Combo { events } => {
                        self.set_key_combo(i, events)?;
                    }
                    Action::Macro { name } => {
                        let m =
                            macros
                                .get(name)
                                .ok_or(crate::Error::InvalidConversion(format!(
                                    "Undefined reference to macro: {name}"
                                )))?;
                        self.set_macro(i, name, &m.events)?;
                        self.write_flash_checked(
                            address::BUTTON_ACTIONS + (i * 4),
                            &[
                                6,
                                i as u8,
                                match m.mode {
                                    MacroMode::Repeat(x) => x as u8,
                                    MacroMode::Toggle => 253,
                                    MacroMode::Hold => 254,
                                    MacroMode::UntilPress => 255,
                                },
                            ],
                        )?;
                        continue;
                    }
                    _ => {}
                }
                self.write_flash_checked(
                    address::BUTTON_ACTIONS + (i * 4),
                    &action_to_raw(action),
                )?;
            }
        }
        Ok(())
    }

    fn key_combo(&self, index: usize) -> crate::Result<Vec<KeyEvent>> {
        assert_range(0..NUM_BUTTONS, index)?;
        let address = address::COMBOS + (index * 32);
        let len = read_flash(&self.device, address, 1)?[0] as usize;
        assert_range(1..=MAX_COMBO_EVENTS, len)?;
        let data = self.read_flash_checked(address, (len * 3) + 1)?;
        let mut key_events = Vec::new();
        for i in 0..len {
            let start_byte = 1 + (i * 3);
            key_events.push(key_event_from_raw(&data[start_byte..(start_byte + 3)])?);
        }
        Ok(key_events)
    }

    fn set_key_combo(&self, index: usize, key_events: &[KeyEvent]) -> crate::Result<()> {
        assert_range(0..NUM_BUTTONS, index)?;
        assert_range(1..=MAX_COMBO_EVENTS, key_events.len())?;
        let mut data = vec![key_events.len() as u8];
        for key_event in key_events {
            data.extend_from_slice(&key_event_to_raw(key_event)?);
        }
        self.write_flash_checked(address::COMBOS + (index * 32), &data)
    }

    fn get_macro(&self, index: usize) -> crate::Result<(String, Vec<MacroEvent>)> {
        assert_range(0..NUM_BUTTONS, index)?;

        let mut address = address::MACROS + (index * 384);
        let name_len = read_flash(&self.device, address, 1)?[0] as usize;
        assert_range(1..=MAX_MACRO_NAME_LEN, name_len)?;
        address += 1;

        let name =
            String::from_utf8_lossy(&read_flash(&self.device, address, name_len)?).to_string();
        address += MAX_MACRO_NAME_LEN;

        let events_len = read_flash(&self.device, address, 1)?[0] as usize;
        assert_range(1..=MAX_MACRO_EVENTS, events_len)?;
        address += 1;

        let events_bytes = read_flash(&self.device, address, events_len * 5)?;

        let mut events = Vec::new();
        for i in (0..(events_len * 5)).step_by(5) {
            let key_event = key_event_from_raw(&events_bytes[i..(i + 3)])?;
            let delay_ms = u16::from_be_bytes([events_bytes[i + 3], events_bytes[i + 4]]);
            events.push(MacroEvent {
                key_event,
                delay_ms,
            });
        }

        Ok((name, events))
    }

    fn set_macro(
        &self,
        index: usize,
        name: &str,
        macro_events: &[MacroEvent],
    ) -> crate::Result<()> {
        assert_range(0..NUM_BUTTONS, index)?;
        let mut address = address::MACROS + (index * 384);

        let mut buf = vec![0];
        buf.extend(name.as_bytes());
        buf[0] = buf.len() as u8 - 1;
        assert_range(1..=MAX_MACRO_NAME_LEN, buf[0])?;
        write_flash(&self.device, address, buf)?;
        address += 31;

        assert_range(1..MAX_MACRO_EVENTS, macro_events.len())?;
        let mut buf = vec![macro_events.len() as u8];
        for event in macro_events {
            buf.extend(key_event_to_raw(&event.key_event)?);
            buf.extend(u16::to_be_bytes(event.delay_ms));
        }

        write_flash(&self.device, address, buf)
    }
}

impl Mouse for Atlantis {
    const NUM_PROFILES: usize = 4;

    fn profile(&self, index: usize) -> crate::Result<Profile> {
        // Only the active profile can be accessed, so store the current profile and
        // switch.
        let active_profile = self.active_profile()?;
        if active_profile != index {
            self.set_active_profile(index)?;
        }

        let (button_map, macros) = self.button_mappings()?;
        let profile = Profile {
            poll_rate: Some(self.poll_rate()?),
            current_resolution_index: Some(self.resolution_index()? as usize),
            resolutions: self.resolutions()?,
            resolution_colors: self.resolution_colors()?,
            charging_color: Some(self.charging_color()?),
            lift_off_distance: Some(self.lift_off_distance()?),
            debounce_ms: Some(self.debounce_ms()?),
            motion_sync: Some(self.motion_sync()?),
            angle_snapping: Some(self.angle_snapping()?),
            ripple_control: Some(self.ripple_control()?),
            peak_performance: Some(self.peak_performance()?),
            peak_performance_time: Some(self.peak_performance_seconds()?),
            high_performance: Some(self.high_performance()?),
            button_map,
            macros,
        };

        // Switch back to original profile.
        if active_profile != index {
            self.set_active_profile(active_profile)?;
        }

        Ok(profile)
    }

    fn set_profile(&self, index: usize, profile: &Profile) -> crate::Result<()> {
        // Only the active profile can be accessed, so store the current profile and
        // switch.
        let active_profile = self.active_profile()?;
        if active_profile != index {
            self.set_active_profile(index)?;
        }

        if let Some(val) = profile.poll_rate {
            self.set_poll_rate(val)?;
        }
        if let Some(val) = profile.current_resolution_index {
            self.set_resolution_index(val as u8)?;
        }
        if !profile.resolutions.is_empty() {
            self.set_resolutions(&profile.resolutions)?;
        }
        if !profile.resolution_colors.is_empty() {
            self.set_resolution_colors(&profile.resolution_colors)?;
        }
        if let Some(val) = profile.charging_color {
            self.set_charging_color(&val)?;
        }
        if let Some(val) = profile.lift_off_distance {
            self.set_lift_off_distance(val as u8)?;
        }
        if let Some(val) = profile.debounce_ms {
            self.set_debounce_ms(val as u8)?;
        }
        if let Some(val) = profile.motion_sync {
            self.set_motion_sync(val)?;
        }
        if let Some(val) = profile.angle_snapping {
            self.set_angle_snapping(val)?;
        }
        if let Some(val) = profile.ripple_control {
            self.set_ripple_control(val)?;
        }
        if let Some(val) = profile.peak_performance {
            self.set_peak_performance(val)?;
        }
        if let Some(val) = profile.peak_performance_time {
            self.set_peak_performance_seconds(val)?;
        }
        if let Some(val) = profile.high_performance {
            self.set_high_performance(val)?;
        }
        if !profile.button_map.is_empty() {
            self.set_button_mappings(&profile.button_map, &profile.macros)?;
        }

        // Switch back to original profile.
        if active_profile != index {
            self.set_active_profile(active_profile)?;
        }

        Ok(())
    }

    fn active_profile(&self) -> crate::Result<usize> {
        read_active_profile(&self.device).map(|p| p as usize)
    }

    fn set_active_profile(&self, profile: usize) -> crate::Result<()> {
        if profile < Self::NUM_PROFILES {
            write_active_profile(&self.device, profile as u8)
        } else {
            Err(crate::Error::InvalidConversion(format!(
                "Profile index '{}' is out of range (0-3)",
                profile
            )))
        }
    }

    fn battery_voltage(&self) -> crate::Result<u16> {
        read_battery_voltage(&self.device)
    }

    fn battery_percentage(&self) -> crate::Result<u8> {
        let battery_mv = self.battery_voltage()?;

        // Basic linear battery percentage calculation.
        let voltage_range = (BATTERY_MAX_MILLIVOLTS - BATTERY_MIN_MILLIVOLTS) as f32;
        let battery_percent =
            battery_mv.saturating_sub(BATTERY_MIN_MILLIVOLTS) as f32 / voltage_range * 100.0;
        Ok((battery_percent.round() as u8).min(100))
    }
}

fn resolution_to_raw(resolution: u16) -> u8 {
    (resolution / 50).saturating_sub(1) as u8
}

fn resolution_from_raw(raw: u8) -> u16 {
    (raw as u16 + 1) * 50
}

fn action_to_raw(action: &Action) -> [u8; 3] {
    match action {
        Action::Disabled => [0, 0, 0],

        Action::LeftClick => [1, 1, 0],
        Action::RightClick => [1, 2, 0],
        Action::MiddleClick => [1, 4, 0],
        Action::BackClick => [1, 8, 0],
        Action::ForwardClick => [1, 16, 0],

        Action::ResolutionLoop => [2, 1, 0],
        Action::ResolutionUp => [2, 2, 0],
        Action::ResolutionDown => [2, 3, 0],
        Action::ResolutionLock { resolution } => [10, resolution_to_raw(*resolution), 0],

        Action::PollRateLoop => [7, 0, 0],

        Action::WheelLeft => [3, 1, 0],
        Action::WheelRight => [3, 2, 0],
        Action::WheelUp => [11, 1, 0],
        Action::WheelDown => [11, 2, 0],

        Action::Fire { interval, repeat } => [4, *interval, *repeat],

        Action::Combo { .. } => [5, 0, 0],
        Action::Macro { .. } => unimplemented!("Macro actions should be converted manually."),
    }
}

fn action_from_raw(raw: &[u8]) -> crate::Result<Action> {
    Ok(match raw {
        [0, 0, 0] => Action::Disabled,

        [1, 1, 0] => Action::LeftClick,
        [1, 2, 0] => Action::RightClick,
        [1, 4, 0] => Action::MiddleClick,
        [1, 8, 0] => Action::BackClick,
        [1, 16, 0] => Action::ForwardClick,

        [2, 1, 0] => Action::ResolutionLoop,
        [2, 2, 0] => Action::ResolutionUp,
        [2, 3, 0] => Action::ResolutionDown,
        [10, raw_resolution, 0] => Action::ResolutionLock {
            resolution: resolution_from_raw(*raw_resolution),
        },

        [7, 0, 0] => Action::PollRateLoop,

        [3, 1, 0] => Action::WheelLeft,
        [3, 2, 0] => Action::WheelRight,
        [11, 1, 0] => Action::WheelUp,
        [11, 2, 0] => Action::WheelDown,

        [4, interval, repeat] => Action::Fire {
            interval: *interval,
            repeat: *repeat,
        },

        _ => {
            return Err(crate::Error::InvalidConversion(format!(
                "Button action data from mouse is invalid: {raw:?}"
            )))
        }
    })
}
fn key_event_to_raw(key_event: &KeyEvent) -> crate::Result<[u8; 3]> {
    let modifier = match key_event.key {
        KeyMappingId::ControlLeft => KeyModifiers::ControlLeft.bits(),
        KeyMappingId::ShiftLeft => KeyModifiers::ShiftLeft.bits(),
        KeyMappingId::AltLeft => KeyModifiers::AltLeft.bits(),
        KeyMappingId::MetaLeft => KeyModifiers::MetaLeft.bits(),
        KeyMappingId::ControlRight => KeyModifiers::ControlRight.bits(),
        KeyMappingId::ShiftRight => KeyModifiers::ShiftRight.bits(),
        KeyMappingId::AltRight => KeyModifiers::AltRight.bits(),
        KeyMappingId::MetaRight => KeyModifiers::MetaRight.bits(),
        _ => 0,
    };

    let mut key_flags = match key_event.state {
        KeyState::Pressed => 0b10000000,
        KeyState::Released => 0b01000000,
    };

    if modifier == 0 {
        // HID code
        key_flags |= 1;
        let code = KeyMap::from(key_event.key).usb.to_le_bytes();
        Ok([key_flags, code[0], code[1]])
    } else {
        // Modifier mask
        Ok([key_flags, modifier, 0])
    }
}

fn key_event_from_raw(raw: &[u8]) -> crate::Result<KeyEvent> {
    let state = match raw[0] & 0b11000000 {
        0b10000000 => KeyState::Pressed,
        0b01000000 => KeyState::Released,
        flags => {
            return Err(crate::Error::InvalidConversion(format!(
                "Invalid key event pressed/released flags: {flags:0>2X}"
            )))
        }
    };
    let code = u16::from_le_bytes([raw[1], raw[2]]);
    let id = match raw[0] & 0b111 {
        // Direction
        0b100 => match code {
            1 => KeyMappingId::ArrowLeft,
            2 => KeyMappingId::ArrowRight,
            // 4 => KeyMappingId::????, // TODO middle direction
            8 => KeyMappingId::ArrowDown,
            16 => KeyMappingId::ArrowUp,
            c => {
                return Err(crate::Error::InvalidConversion(format!(
                    "Unknown key direction code: {c}"
                )))
            }
        },

        // HID consumer control
        0b010 => {
            return Err(crate::Error::InvalidConversion(
                "HID consumer control codes are unimplemented.".to_string(),
            ))
        }

        // HID
        0b001 => {
            // USB HID usage page 7 for keyboards.
            KeyMap::from_usb_code(7, code)
                .map_err(|_| {
                    crate::Error::InvalidConversion(format!(
                        "Failed to convert from raw HID code: {code}"
                    ))
                })?
                .id
        }

        // Modifier mask
        0b000 => match KeyModifiers::from_bits(code as u8).ok_or(
            crate::Error::InvalidConversion(format!("Invalid modifier mask from raw: {code}")),
        )? {
            KeyModifiers::ControlLeft => KeyMappingId::ControlLeft,
            KeyModifiers::ShiftLeft => KeyMappingId::ShiftLeft,
            KeyModifiers::AltLeft => KeyMappingId::AltLeft,
            KeyModifiers::MetaLeft => KeyMappingId::MetaLeft,
            KeyModifiers::ControlRight => KeyMappingId::ControlRight,
            KeyModifiers::ShiftRight => KeyMappingId::ShiftRight,
            KeyModifiers::AltRight => KeyMappingId::AltRight,
            KeyModifiers::MetaRight => KeyMappingId::MetaRight,

            // Multiple modifiers might work, but we'd need to figure out how to represent
            // that in RON/JSON serialized profiles.
            _ => {
                return Err(crate::Error::InvalidConversion(format!(
                    "Only one modifier is currently supported in key events."
                )));
            }
        },

        bits => {
            return Err(crate::Error::InvalidConversion(format!(
                "Invalid key event type flags: {bits}"
            )))
        }
    };

    Ok(KeyEvent { key: id, state })
}

fn checksum(data: &[u8]) -> u8 {
    let mut sum: u8 = 171;
    for d in data {
        sum = sum.wrapping_add(*d);
    }
    0u8.wrapping_sub(sum)
}

fn assert_range<
    T: PartialOrd + PartialEq + std::fmt::Display,
    U: Into<T>,
    R: RangeBounds<T> + std::fmt::Debug,
>(
    range: R,
    val: U,
) -> crate::Result<()> {
    let val = val.into();
    if range.contains(&val) {
        Ok(())
    } else {
        Err(crate::Error::InvalidConversion(format!(
            "Value '{val}' out of range '{range:?}'",
        )))
    }
}
