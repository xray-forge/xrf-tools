//! What an open level reads beside its own bundle, each kept as a part of its session with what it was read from, so a
//! change to that moves the part and not the level. One part type to a file, owning how its value is read and kept,
//! its value beside it: read from the level's own files (`level`), from the game's spawn and what it names (`spawn`),
//! for what the renderer draws beside the sectors (`drawn`), from the game's environment (`environment`), and from the
//! game's console defaults (`console`). Code names a part by its type's `SessionPart::NAME`; each file's `LEVEL_*_PART`
//! constant is that name as the generated bindings carry it.

mod console;
mod drawn;
mod environment;
mod level;
mod spawn;

pub(crate) use console::LevelConsolePart;
pub(crate) use drawn::{
  LevelDetailsPart, LevelLightsPart, LevelObjectMotionsPart, LevelParticlesPart, PackedLevelDetails, PackedLevelLights,
  PackedLevelParticles,
};
pub(crate) use environment::{LevelEnvironment, LevelEnvironmentPart, LevelWeatherPart};
pub(crate) use level::{
  LevelCollisionPart, LevelGeometryPart, LevelMapPart, LevelPlacementsPart, LevelSurfacesPart, LevelTexturesPart,
};
pub use level::{LevelMapArea, LevelMapDescription, LevelPlacements, LevelTextureReference};
pub(crate) use spawn::{
  LevelGraphPart, LevelHemiPart, LevelSectionsPart, LevelSpawn, LevelSpawnPart, LevelSpawnVisual, LevelSpawnVisuals,
  LevelSpawnVisualsPart, LevelStartPart, SpawnVisualReader,
};
pub use spawn::{LevelSpawnObjectHemi, LevelStart, LevelStartOrigin};
