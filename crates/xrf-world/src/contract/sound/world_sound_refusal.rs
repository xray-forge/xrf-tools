use serde::{Deserialize, Serialize};

use crate::contract::sound::world_sound_kind::WorldSoundKind;

/// A sound a scene could not play, and why: its file is missing, it cannot be decoded, or no voice was left for it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundRefusal {
  /// The sound's reference.
  pub sound: String,
  /// What would have played it, none where the audio refused it by its sound alone.
  pub kind: Option<WorldSoundKind>,
  pub reason: String,
}
