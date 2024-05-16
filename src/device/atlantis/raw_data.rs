//! Lamzu-Atlantis-specific profile data types.

use crate::data::*;
use binrw::binrw;
use keycode::{KeyMap, KeyMapping, KeyMappingId, KeyModifiers, KeyState};

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Copy)]
pub enum RawAction {
    #[brw(magic = 0x00u8)]
    Disabled,

    #[brw(magic = 0x01u8)]
    Button { id: u8 },

    #[brw(magic = 0x0201u16)]
    DpiLoop,

    #[brw(magic = 0x0202u16)]
    DpiUp,

    #[brw(magic = 0x0203u16)]
    DpiDown,

    #[brw(magic = 0x0301u16)]
    WheelLeft,

    #[brw(magic = 0x0302u16)]
    WheelRight,

    #[brw(magic = 0x04u8)]
    Fire { interval: u8, repeat: u8 },

    #[brw(magic = 0x05u8)]
    Combo,

    #[brw(magic = 0x06u8)]
    Macro { index: u8 },

    #[brw(magic = 0x07u8)]
    PollRateLoop,

    #[brw(magic = 0x0au8)]
    DpiLock { dpi: u8 },

    #[brw(magic = 0x0b01u16)]
    WheelUp,

    #[brw(magic = 0x0b02u16)]
    WheelDown,
}

impl From<PaddedRawAction> for RawAction {
    fn from(padded: PaddedRawAction) -> Self {
        padded.action
    }
}

/// Wraps an action to pad to 3 bytes long. Necessary since binrw padding is
/// unsupported on enum definitions.
#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct PaddedRawAction {
    #[brw(pad_size_to = 3)]
    pub action: RawAction,
}

impl From<RawAction> for PaddedRawAction {
    fn from(action: RawAction) -> Self {
        Self { action }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawDirection {
    #[brw(magic = 1u16)]
    Left,

    #[brw(magic = 2u16)]
    Right,

    #[brw(magic = 4u16)]
    Middle,

    #[brw(magic = 8u16)]
    Back,

    #[brw(magic = 16u16)]
    Forward,
}

impl From<RawDirection> for KeyMappingId {
    fn from(raw_direction: RawDirection) -> Self {
        match raw_direction {
            RawDirection::Left => Self::ArrowLeft,
            RawDirection::Right => Self::ArrowRight,
            RawDirection::Middle => Self::None,
            RawDirection::Back => Self::ArrowDown,
            RawDirection::Forward => Self::ArrowUp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawKeyId {
    Modifier(u16),
    Hid(u16),
    Consumer(u16),
    Direction(RawDirection),
}

impl From<KeyMappingId> for RawKeyId {
    fn from(key_mapping_id: KeyMappingId) -> Self {
        let modifier = match key_mapping_id {
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

        if modifier == 0 {
            Self::Hid(KeyMap::from(key_mapping_id).usb)
        } else {
            Self::Modifier(modifier as u16)
        }
    }
}

impl TryFrom<RawKeyId> for KeyMappingId {
    type Error = ();

    fn try_from(raw_key_id: RawKeyId) -> Result<Self, Self::Error> {
        Ok(match raw_key_id {
            RawKeyId::Modifier(modifier) => {
                match KeyModifiers::from_bits(modifier as u8).ok_or(())? {
                    KeyModifiers::ControlLeft => Self::ControlLeft,
                    KeyModifiers::ShiftLeft => Self::ShiftLeft,
                    KeyModifiers::AltLeft => Self::AltLeft,
                    KeyModifiers::MetaLeft => Self::MetaLeft,
                    KeyModifiers::ControlRight => Self::ControlRight,
                    KeyModifiers::ShiftRight => Self::ShiftRight,
                    KeyModifiers::AltRight => Self::AltRight,
                    KeyModifiers::MetaRight => Self::MetaRight,

                    // Lamzu desktop software only allows one modifier per event. Error for no
                    // modifier / multiple modifiers.
                    _ => return Err(()),
                }
            }

            RawKeyId::Hid(keycode) => KeyMap::try_from(KeyMapping::Usb(keycode))?.id,

            RawKeyId::Consumer(_) => return Err(()), // TODO: Implement consumer control codes.

            RawKeyId::Direction(direction) => direction.into(),
        })
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawKeyEvent {
    #[brw(magic = 0x80u8)]
    PressModifier(#[brw(little)] u16),

    #[brw(magic = 0x81u8)]
    PressHid(#[brw(little)] u16),

    #[brw(magic = 0x82u8)]
    PressConsumer(#[brw(little)] u16),

    #[brw(magic = 0x84u8)]
    PressDirection(RawDirection),

    #[brw(magic = 0x40u8)]
    ReleaseModifier(#[brw(little)] u16),

    #[brw(magic = 0x41u8)]
    ReleaseHid(#[brw(little)] u16),

    #[brw(magic = 0x42u8)]
    ReleaseConsumer(#[brw(little)] u16),

    #[brw(magic = 0x44u8)]
    ReleaseDirection(RawDirection),
}

impl RawKeyEvent {
    /// Splits `RawKeyEvent` into a `KeyState` and `RawKeyId`.
    fn split(self) -> (KeyState, RawKeyId) {
        match self {
            Self::PressModifier(modifier) => (KeyState::Pressed, RawKeyId::Modifier(modifier)),
            Self::PressHid(hid) => (KeyState::Pressed, RawKeyId::Hid(hid)),
            Self::PressConsumer(consumer) => (KeyState::Pressed, RawKeyId::Consumer(consumer)),
            Self::PressDirection(direction) => (KeyState::Pressed, RawKeyId::Direction(direction)),
            Self::ReleaseModifier(modifier) => (KeyState::Released, RawKeyId::Modifier(modifier)),
            Self::ReleaseHid(hid) => (KeyState::Released, RawKeyId::Hid(hid)),
            Self::ReleaseConsumer(consumer) => (KeyState::Released, RawKeyId::Consumer(consumer)),
            Self::ReleaseDirection(direction) => {
                (KeyState::Released, RawKeyId::Direction(direction))
            }
        }
    }

    /// Creates a `RawKeyEvent` from a `KeyState` and a `RawKeyId`.
    fn join(key_state: KeyState, raw_key_id: RawKeyId) -> Self {
        match key_state {
            KeyState::Pressed => match raw_key_id {
                RawKeyId::Modifier(modifier) => Self::PressModifier(modifier),
                RawKeyId::Hid(hid) => Self::PressHid(hid),
                RawKeyId::Consumer(consumer) => Self::PressConsumer(consumer),
                RawKeyId::Direction(direction) => Self::PressDirection(direction),
            },
            KeyState::Released => match raw_key_id {
                RawKeyId::Modifier(modifier) => Self::ReleaseModifier(modifier),
                RawKeyId::Hid(hid) => Self::ReleaseHid(hid),
                RawKeyId::Consumer(consumer) => Self::ReleaseConsumer(consumer),
                RawKeyId::Direction(direction) => Self::ReleaseDirection(direction),
            },
        }
    }
}

impl From<KeyEvent> for RawKeyEvent {
    fn from(key_event: KeyEvent) -> Self {
        Self::join(key_event.state.into(), key_event.key.into())
    }
}

impl TryFrom<RawKeyEvent> for KeyEvent {
    type Error = crate::Error;

    fn try_from(raw_key_event: RawKeyEvent) -> crate::Result<Self> {
        let (state, raw_key_id) = raw_key_event.split();
        Ok(Self {
            state,
            key: raw_key_id.try_into().map_err(|_| {
                crate::Error::InvalidConversion(format!(
                    "Failed to convert from raw key id: {:?}",
                    raw_key_id
                ))
            })?,
        })
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct RawMacroEvent {
    key_event: RawKeyEvent,

    #[brw(big)]
    delay_ms: u16,
}

impl From<MacroEvent> for RawMacroEvent {
    fn from(macro_event: MacroEvent) -> Self {
        Self {
            key_event: macro_event.key_event.into(),
            delay_ms: macro_event.delay_ms,
        }
    }
}

impl TryFrom<RawMacroEvent> for MacroEvent {
    type Error = crate::Error;

    fn try_from(raw_macro_event: RawMacroEvent) -> crate::Result<Self> {
        Ok(Self {
            key_event: raw_macro_event.key_event.try_into()?,
            delay_ms: raw_macro_event.delay_ms,
        })
    }
}

/// A named sequence of up to 70 key events with delays.
#[binrw]
#[derive(Debug, Clone)]
pub struct RawMacro {
    #[br(temp)]
    #[bw(try_calc(u8::try_from(name.len())))]
    name_len: u8,

    #[br(count = name_len)]
    #[brw(pad_size_to = 30, assert(name.len() <= 30))]
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    #[bw(map = |s: &String| s.as_bytes())]
    pub name: String,

    #[br(temp)]
    #[bw(try_calc(u8::try_from(events.len())))]
    events_len: u8,

    #[br(count = events_len)]
    #[brw(assert(events.len() <= 70))]
    pub events: Vec<RawMacroEvent>,
}

impl RawMacro {
    pub fn new(name: String, events: Vec<RawMacroEvent>) -> Self {
        Self { name, events }
    }
}

/// A short sequence of up to 3 keys (6 events).
#[binrw]
#[derive(Debug, Clone)]
pub struct RawCombo {
    #[br(temp)]
    #[bw(try_calc(u8::try_from(events.len())))]
    events_len: u8,

    #[br(count = events_len)]
    #[brw(assert(events.len() <= 6))]
    events: Vec<RawKeyEvent>,
}

impl From<Vec<KeyEvent>> for RawCombo {
    fn from(key_events: Vec<KeyEvent>) -> Self {
        Self {
            events: key_events
                .iter()
                .map(|event| event.clone().into())
                .collect(),
        }
    }
}

impl TryFrom<RawCombo> for Vec<KeyEvent> {
    type Error = crate::Error;

    fn try_from(raw_combo: RawCombo) -> crate::Result<Self> {
        raw_combo
            .events
            .iter()
            .cloned()
            .map(KeyEvent::try_from)
            .collect()
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, Copy)]
pub struct RawDpi {
    dpi_x: u8,

    #[brw(pad_after = 1)]
    dpi_y: u8,
}

impl From<Dpi> for RawDpi {
    fn from(dpi: Dpi) -> Self {
        match dpi {
            Dpi::Linked(dpi_xy) => {
                let raw = dpi_to_raw(dpi_xy);
                Self {
                    dpi_x: raw,
                    dpi_y: raw,
                }
            }
            Dpi::Independent(dpi_x, dpi_y) => Self {
                dpi_x: dpi_to_raw(dpi_x),
                dpi_y: dpi_to_raw(dpi_y),
            },
        }
    }
}

impl From<RawDpi> for Dpi {
    fn from(raw_dpi: RawDpi) -> Dpi {
        let dpi_x = dpi_from_raw(raw_dpi.dpi_x);
        let dpi_y = dpi_from_raw(raw_dpi.dpi_y);
        if dpi_x == dpi_y {
            Dpi::Linked(dpi_x)
        } else {
            Dpi::Independent(dpi_x, dpi_y)
        }
    }
}

pub fn dpi_to_raw(dpi: u16) -> u8 {
    (dpi / 50).saturating_sub(1) as u8
}

pub fn dpi_from_raw(raw: u8) -> u16 {
    (raw as u16 + 1) * 50
}

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct RawColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl From<Color> for RawColor {
    fn from(color: Color) -> Self {
        Self {
            red: color.red,
            green: color.green,
            blue: color.blue,
        }
    }
}

impl From<RawColor> for Color {
    fn from(raw_color: RawColor) -> Self {
        Self {
            red: raw_color.red,
            green: raw_color.green,
            blue: raw_color.blue,
        }
    }
}
