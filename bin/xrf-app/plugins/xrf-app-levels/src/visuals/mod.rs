//! Spawned objects' visuals, shared by a compiled level's spawn and a scene's spawn points: each visual packed, posed
//! and dressed, the per-session cache, and its reader.

mod spawn_visual;
mod spawn_visual_reader;
mod spawn_visuals;

pub(crate) use spawn_visual::SpawnVisual;
pub(crate) use spawn_visual_reader::SpawnVisualReader;
pub(crate) use spawn_visuals::SpawnVisuals;
