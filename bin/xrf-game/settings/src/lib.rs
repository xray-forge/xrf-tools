#![doc = include_str!("../README.md")]

pub(crate) mod game_preset;
pub(crate) mod game_setting_override;
pub(crate) mod game_settings;
pub(crate) mod game_settings_load;
pub(crate) mod game_settings_merge;
pub(crate) mod game_window_mode;
pub(crate) mod game_window_settings;
#[cfg(test)]
mod tests;

pub use crate::game_preset::GamePreset;
pub use crate::game_settings::GameSettings;
pub use crate::game_window_mode::GameWindowMode;
pub use crate::game_window_settings::GameWindowSettings;
