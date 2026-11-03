use std::sync::Arc;

use crate::host::sound::world_sound_info::WorldSoundInfo;

/// A sound a level plays by its reference, `sounds\` relative and without its extension, and what its file says of it:
/// none where the file is missing or not a sound the engine loads, which plays nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldSound {
  pub reference: Arc<str>,
  pub info: Option<WorldSoundInfo>,
}

impl WorldSound {
  pub fn new(reference: &str, info: Option<WorldSoundInfo>) -> Self {
    Self {
      reference: Arc::from(reference),
      info,
    }
  }
}
