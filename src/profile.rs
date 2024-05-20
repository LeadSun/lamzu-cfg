use crate::data::{Action, Color, Dpi, MacroEvent};
use std::collections::HashMap;

/// Standard profile data for mice.
///
/// All fields are optional to allow for partial profile writes.
#[derive(Debug, Clone, Default)]
pub struct Profile {
    pub poll_rate: Option<u16>,
    pub current_dpi_index: Option<usize>,
    pub lift_off_distance: Option<u8>,
    pub debounce_ms: Option<u8>,
    pub motion_sync: Option<bool>,
    pub angle_snapping: Option<bool>,
    pub ripple_control: Option<bool>,
    pub peak_performance: Option<bool>,
    pub peak_performance_time: Option<u16>,
    pub high_performance: Option<bool>,
    pub dpis: Vec<Dpi>,
    pub dpi_colors: Vec<Color>,
    pub button_actions: Vec<Action>,
    pub macros: HashMap<String, Vec<MacroEvent>>,
}
