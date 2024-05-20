use crate::data::{Action, Color, Dpi, MacroEvent};
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
    pub current_dpi_index: Option<usize>,

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
    pub dpis: Vec<Dpi>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dpi_colors: Vec<Color>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub button_actions: Vec<Action>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub macros: HashMap<String, Vec<MacroEvent>>,
}
