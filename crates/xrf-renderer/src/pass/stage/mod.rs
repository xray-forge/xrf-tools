//! The passes a level's frame runs, built once for every view and grouped by the part of the frame they draw.

pub(crate) mod effect_passes;
pub(crate) mod lighting_passes;
pub(crate) mod pass_build;
pub(crate) mod scene_passes;
pub(crate) mod tracing_passes;
pub(crate) mod weather_passes;
