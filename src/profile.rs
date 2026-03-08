use keycode::{KeyMappingId, KeyState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard profile data for mice.
///
/// All fields are optional to allow for partial profile writes.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Profile {
    // Ugly and verbose, but necessary to auto wrap, unwrap, and skip options.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub poll_rate: Option<u16>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub current_resolution_index: Option<usize>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub lift_off_distance: Option<u8>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub debounce_ms: Option<u8>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub motion_sync: Option<bool>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub angle_snapping: Option<bool>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub ripple_control: Option<bool>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub peak_performance: Option<bool>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub peak_performance_time: Option<u16>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::unwrap_or_skip"
    )]
    pub high_performance: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolutions: Vec<Resolution>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolution_colors: Vec<Color>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charging_color: Option<Color>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub button_map: HashMap<Button, Action>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub macros: HashMap<String, Macro>,
}

/// Physical buttons on the mouse.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Button {
    Left,
    Middle,
    Right,
    Forward,
    Back,
    Bottom,
}

/// Mouse actions that can be mapped to buttons.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Action {
    Disabled,

    LeftClick,
    RightClick,
    MiddleClick,
    BackClick,
    ForwardClick,

    ResolutionLoop,
    ResolutionUp,
    ResolutionDown,
    ResolutionLock { resolution: u16 },

    PollRateLoop,

    WheelLeft,
    WheelRight,
    WheelUp,
    WheelDown,

    Fire { interval: u8, repeat: u8 },

    Combo { events: Vec<KeyEvent> },
    Macro { name: String },
}

/// Key pressed / released events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct KeyEvent {
    pub key: KeyMappingId,
    pub state: KeyState,
}

/// Sequence of key presses that can be triggered by a button.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Macro {
    pub mode: MacroMode,
    pub events: Vec<MacroEvent>,
}

/// Key pressed / released events with a delay.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct MacroEvent {
    pub key_event: KeyEvent,
    pub delay_ms: u16,
}

/// Macro repeat behaviour.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum MacroMode {
    /// Repeat x times.
    Repeat(u8),

    /// Repeat until the same button is pressed again.
    Toggle,

    /// Repeat while the button is held.
    Hold,

    /// Repeate until any button is pressed.
    UntilPress,
}

/// XY DPI.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Resolution {
    pub x: u16,
    pub y: u16,
}

impl Resolution {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            red: bytes[0],
            green: bytes[1],
            blue: bytes[2],
        }
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        [self.red, self.green, self.blue]
    }
}
