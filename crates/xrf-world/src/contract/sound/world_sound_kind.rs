use serde::{Deserialize, Serialize};

use crate::contract::sound::world_sound_category::WorldSoundCategory;

/// What plays a sound in a level.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorldSoundKind {
  /// One of the level's static sounds, `level.snd_static`.
  Static,
  /// One of the level's music tracks.
  Music,
  /// A sound channel of the weather's ambient, around the camera.
  Channel,
  /// The sound an ambient effect plays where it starts.
  AmbientEffect,
  /// The rain's loop.
  Rain,
  /// A thunderbolt's strike.
  Thunder,
}

impl WorldSoundKind {
  /// The volume it is heard at, each with its own.
  pub fn get_category(self) -> WorldSoundCategory {
    match self {
      Self::Music => WorldSoundCategory::Music,
      Self::Static | Self::Channel | Self::AmbientEffect | Self::Rain | Self::Thunder => WorldSoundCategory::Ambience,
    }
  }
}
