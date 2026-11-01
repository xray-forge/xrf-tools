//! What a level's frame draws, one folder a group of passes, named for the part of the frame they draw. A group holds
//! its passes, built once for every view, its view stage where it has one, and a folder a feature: its unit, passes,
//! parameters, uniforms and the other types it owns.

pub(crate) mod effects;
pub(crate) mod image;
pub(crate) mod lighting;
pub(crate) mod tracing;
pub(crate) mod weather;
