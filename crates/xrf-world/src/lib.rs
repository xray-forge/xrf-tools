//! The world a renderer draws: what moves and plays in a level whatever draws it — the cameras, the weather, the
//! level's streaming into its scene and its skinned objects' poses — run inline before each frame and handed to the
//! renderer as one frame's input.

pub(crate) mod camera;
pub(crate) mod level;
pub(crate) mod weather;
pub(crate) mod world;
pub(crate) mod world_viewport;

#[cfg(test)]
mod tests;

pub use crate::world::World;
