use crate::contract::sound::world_sound_kind::WorldSoundKind;

/// A sound the host asks a level to play once, as a game plays its actor's footsteps: by its reference (`sounds\`
/// relative, no extension), which the level's source reads once and keeps.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldSoundCue {
  pub reference: String,
  pub kind: WorldSoundKind,
  /// In engine space; unread for a sound played 2D.
  pub position: [f32; 3],
  /// Whether it plays from the listener, as X-Ray plays the actor's own steps in first person (`sm_2D`).
  pub is_2d: bool,
  pub volume: f32,
}
