use serde::{Deserialize, Serialize};

use crate::contract::sound::world_sound_channel_report::WorldSoundChannelReport;
use crate::contract::sound::world_sound_music_report::WorldSoundMusicReport;
use crate::contract::sound::world_sound_refusal::WorldSoundRefusal;
use crate::contract::sound::world_sound_voice_report::WorldSoundVoiceReport;

/// What a viewport's scene plays and why: each sound and how it is heard, the music, the ambient's channels, and what
/// could not play. The world fills what it schedules; the host's audio fills the gains and its own refusals.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundReport {
  /// Whether the scene is heard: its viewport, or another showing it, is the one listening.
  pub is_audible: bool,
  /// Where it is heard from, in engine space.
  pub listener: [f32; 3],
  /// Game time of day the schedules read, in milliseconds.
  pub game_time: u32,
  pub voices: Vec<WorldSoundVoiceReport>,
  pub music: Option<WorldSoundMusicReport>,
  /// The ambient whose channels play, none while the weather names none.
  pub ambient: Option<String>,
  pub channels: Vec<WorldSoundChannelReport>,
  pub refused: Vec<WorldSoundRefusal>,
  /// Voices realized on the device, and the engine's targets for them.
  pub real_voices: u32,
  pub targets: u32,
  /// The mixer's share of the device's period, as last measured.
  pub mixer_load: Option<f32>,
}
