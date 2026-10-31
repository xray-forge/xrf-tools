use serde::{Deserialize, Serialize};

/// Which of the level's companion files a placement is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelPlacementFile {
  /// `level.ps_static`.
  Particles,
  /// `level.snd_static`.
  Sounds,
  /// `level.snd_env`.
  SoundEnvironments,
  /// `level.fog_vol`.
  FogVolumes,
  /// `level.env_mod`.
  EnvModifiers,
  /// `level.game`'s respawn points.
  RespawnPoints,
}
