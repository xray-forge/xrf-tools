//! What spawned objects' visuals are read as wherever the plugin draws them, a compiled level's spawn and a scene's
//! spawn points alike: each visual packed, posed and dressed, the visuals a session keeps, and the reader that fills
//! them through the search its session hands it.

mod spawn_visual;
mod spawn_visual_reader;
mod spawn_visuals;

pub(crate) use spawn_visual::SpawnVisual;
pub(crate) use spawn_visual_reader::SpawnVisualReader;
pub(crate) use spawn_visuals::SpawnVisuals;
