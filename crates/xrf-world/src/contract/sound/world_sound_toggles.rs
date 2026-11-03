use serde::{Deserialize, Serialize};

use crate::contract::sound::world_sound_category::WorldSoundCategory;

/// What of a viewport's level is heard while it listens: its sound at all, and each category. A sound switched off is
/// silent and its schedule runs on.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundToggles {
  pub is_heard: bool,
  pub is_music: bool,
  pub is_ambience: bool,
  pub is_world: bool,
}

impl Default for WorldSoundToggles {
  fn default() -> Self {
    Self {
      is_heard: true,
      is_music: true,
      is_ambience: true,
      is_world: true,
    }
  }
}

impl WorldSoundToggles {
  /// Whether a category is heard.
  pub fn is_heard(&self, category: WorldSoundCategory) -> bool {
    self.is_heard
      && match category {
        WorldSoundCategory::Music => self.is_music,
        WorldSoundCategory::Ambience => self.is_ambience,
        WorldSoundCategory::World => self.is_world,
      }
  }
}
