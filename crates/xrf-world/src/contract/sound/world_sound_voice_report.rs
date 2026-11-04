use serde::{Deserialize, Serialize};

use crate::contract::sound::world_sound_category::WorldSoundCategory;
use crate::contract::sound::world_sound_kind::WorldSoundKind;

/// One sound a scene plays and how it is heard.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundVoiceReport {
  /// The emitter's id, stable while the same play goes on.
  #[cfg_attr(feature = "typescript-bindings", specta(type = f64))]
  pub id: u64,
  /// The sound's reference.
  pub sound: String,
  pub kind: WorldSoundKind,
  pub category: WorldSoundCategory,
  /// In engine space; from the listener for a 2D sound.
  pub position: [f32; 3],
  pub is_2d: bool,
  /// Metres from the listener; none for a 2D sound.
  pub distance: Option<f32>,
  /// Its linear gain after its volume, fade and distance, before walls, none until the audio has taken it.
  pub gain: Option<f32>,
  /// The share of it heard unobstructed by the level's walls, from 0 to 1, none until the audio has taken it.
  pub occlusion: Option<f32>,
  /// What the walls in its way let through in each band (below 800 Hz, to 8 kHz, above), from 0 to 1.
  pub transmission: Option<[f32; 3]>,
  /// How much of it the listener's reverb hears.
  pub reverb_send: Option<f32>,
  /// Whether it reaches the ears through the HRTF rather than panned.
  pub is_binaural: bool,
  /// Whether it holds one of the engine's targets, so plays on the device, rather than only simulating.
  pub is_real: bool,
  /// Whether its sound is still being read.
  pub is_loading: bool,
  /// Times the audio started its voice over where its clock says, having starved or ended early.
  pub restarts: u32,
}
