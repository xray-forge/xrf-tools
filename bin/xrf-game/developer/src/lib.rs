#![doc = include_str!("../README.md")]

pub(crate) mod console;
pub(crate) mod input;
pub(crate) mod layer;
pub(crate) mod paint;
pub(crate) mod readout;

#[cfg(test)]
mod tests;

pub use crate::console::developer_console::DeveloperConsole;
pub use crate::console::developer_suggestion::DeveloperSuggestion;
pub use crate::layer::developer_frame::DeveloperFrame;
pub use crate::layer::developer_layer::DeveloperLayer;
pub use crate::layer::developer_view::DeveloperView;
pub use crate::readout::developer_readout::DeveloperReadout;
