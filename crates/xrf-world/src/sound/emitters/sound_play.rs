use std::time::Duration;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_sound::WorldSound;

/// How one sound starts.
pub struct SoundPlay<'a> {
  pub sound: &'a WorldSound,
  pub kind: WorldSoundKind,
  /// The level object it plays at, by the id its helper carries.
  pub object: Option<u32>,
  /// In engine space; from the listener for a sound played 2D.
  pub position: [f32; 3],
  /// When it starts playing, on the level's clock.
  pub start: Duration,
  /// How far into its sound it starts, at its own speed.
  pub offset: Duration,
  pub is_looped: bool,
  /// Whether it plays 2D whatever its file, `sm_2D`.
  pub is_2d: bool,
  pub volume: f32,
  pub frequency: f32,
  /// Metres it is heard within, in place of its file's, `set_range`.
  pub range: Option<[f32; 2]>,
}

impl<'a> SoundPlay<'a> {
  /// A sound played once at its own volume and speed, 3D unless its file is stereo.
  pub fn new(sound: &'a WorldSound, kind: WorldSoundKind, position: [f32; 3], start: Duration) -> Self {
    Self {
      sound,
      kind,
      object: None,
      position,
      start,
      offset: Duration::ZERO,
      is_looped: false,
      is_2d: false,
      volume: 1.0,
      frequency: 1.0,
      range: None,
    }
  }
}
