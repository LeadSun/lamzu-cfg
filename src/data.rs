//! Standard mouse configuration data types.

use keycode::{KeyMappingId, KeyState};

/// Mouse actions that can be mapped to buttons.
#[derive(Debug, Clone)]
pub enum Action {
    Disabled,

    LeftClick,
    RightClick,
    MiddleClick,
    BackClick,
    ForwardClick,

    DpiLoop,
    DpiUp,
    DpiDown,
    DpiLock { dpi: u16 },

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: KeyMappingId,
    pub state: KeyState,
}

/// Key pressed / released events with a delay.
#[derive(Debug, Clone, Copy)]
pub struct MacroEvent {
    pub key_event: KeyEvent,
    pub delay_ms: u16,
}

/// Mouse resolution.
#[derive(Debug, Clone, Copy)]
pub enum Dpi {
    /// Both x and y DPI are the same.
    Linked(u16),

    /// Separate x and y DPI.
    Independent(u16, u16),
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
