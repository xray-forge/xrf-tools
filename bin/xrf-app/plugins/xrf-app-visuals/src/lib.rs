#![doc = include_str!("../README.md")]

mod browse;
mod commands;
pub mod ipc;
mod model;
pub mod plugin;
mod pose;
#[cfg(test)]
mod tests;

pub use browse::VisualBrowses;
pub use model::{SelectedVisual, SelectedVisualDescription, VisualModels, VisualRenderSource};
pub use pose::{PosedVisualMotion, VisualPoses};
