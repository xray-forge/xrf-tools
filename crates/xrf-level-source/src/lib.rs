#![doc = include_str!("../README.md")]

pub(crate) mod console;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub(crate) mod helpers;
pub(crate) mod level;
pub(crate) mod level_source_result;
pub(crate) mod objects;
pub(crate) mod parts;
pub(crate) mod render;
pub(crate) mod report;
pub(crate) mod sound;
pub(crate) mod spawn;
#[cfg(test)]
mod tests;
pub(crate) mod visuals;
pub(crate) mod weather;

pub use crate::console::LevelConsoleDefaults;
pub use crate::helpers::{HelperColor, HelperLinkLines};
pub use crate::level::{
  LevelBundle, LevelConfigs, LevelEntry, LevelMount, LevelSource, LevelSunDescription, OpenedLevel, SelectedLevel,
  SelectedLevelDescription,
};
pub use crate::level_source_result::LevelSourceResult;
pub use crate::objects::{
  LEVEL_OBJECT_KINDS, LEVEL_SOUND_EMITTERS_HELPER_KIND, LevelObject, LevelObjectGroup, LevelObjectId, LevelObjectKind,
  LevelObjectKindEntry, LevelObjectPatrol, LevelObjectPatrolPoint, LevelObjectPlacement, LevelObjectShape,
  LevelObjectSources, LevelObjectVolume, LevelObjectsDescription, LevelPatrolLink, LevelPlacementDetails,
  LevelPlacementFile, LevelRestriction, LevelSoundWindow,
};
pub use crate::parts::{
  LevelAiMapPart, LevelCollisionPart, LevelConsolePart, LevelDetailsPart, LevelEnvironment, LevelEnvironmentPart,
  LevelGeometryPart, LevelGraphPart, LevelHemiPart, LevelLightsPart, LevelMapArea, LevelMapDescription, LevelMapPart,
  LevelObjectMotionsPart, LevelParticlesPart, LevelPlacements, LevelPlacementsPart, LevelSectionsPart, LevelSpawn,
  LevelSpawnObjectHemi, LevelSpawnPart, LevelSpawnVisualsPart, LevelStart, LevelStartOrigin, LevelStartPart,
  LevelSurfacesPart, LevelTextureReference, LevelTexturesPart, LevelWeatherPart, NamedSpawnVisual, PackedLevelDetails,
  PackedLevelLights, PackedLevelParticles,
};
pub use crate::render::{LevelAudio, LevelParticleCollider, LevelRenderParts, LevelRenderSource};
pub use crate::report::LevelReport;
pub use crate::sound::{BurstSoundFrame, BurstSoundTee, LevelSoundReport};
pub use crate::spawn::LevelSpawnObjectDetails;
pub use crate::visuals::{SpawnVisual, SpawnVisualReader, SpawnVisuals};
pub use crate::weather::{EnvironmentCycleEntry, LevelWeatherCycle, LevelWeatherDescription, LevelWeatherTexture};
