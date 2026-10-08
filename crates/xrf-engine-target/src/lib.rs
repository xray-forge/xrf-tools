#![doc = include_str!("../README.md")]

mod detection;
mod xray_engine;
mod xray_engine_choice;
mod xray_engine_evidence;
mod xray_engine_resolution;

pub use crate::detection::{WEATHER_GRAPHS_LOGICAL_PATH, detect_engine};
pub use crate::xray_engine::XrayEngine;
pub use crate::xray_engine_choice::XrayEngineChoice;
pub use crate::xray_engine_evidence::XrayEngineEvidence;
pub use crate::xray_engine_resolution::XrayEngineResolution;
