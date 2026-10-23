#![doc = include_str!("../README.md")]

pub(crate) mod ppe_color;
pub(crate) mod ppe_color_map;
pub(crate) mod ppe_file;

#[cfg(test)]
mod tests;

pub use crate::ppe_color::PpeColor;
pub use crate::ppe_color_map::PpeColorMap;
pub use crate::ppe_file::{PPE_COLOR_CHANNELS, PPE_COLOR_MAP_INFLUENCE, PPE_COLORS, PPE_VALUES, PpeFile};
