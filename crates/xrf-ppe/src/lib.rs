#![doc = include_str!("../README.md")]

pub(crate) mod ppe_color;
pub(crate) mod ppe_color_map;
pub(crate) mod ppe_file;
pub(crate) mod ppe_file_evaluation;
pub(crate) mod ppe_screen;

#[cfg(test)]
mod tests;

pub use crate::ppe_color::PpeColor;
pub use crate::ppe_color_map::PpeColorMap;
pub use crate::ppe_file::PpeFile;
pub use crate::ppe_screen::PpeScreen;
