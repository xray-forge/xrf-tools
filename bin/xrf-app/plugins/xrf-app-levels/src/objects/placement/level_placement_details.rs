use serde::Serialize;

use crate::objects::LevelSoundWindow;

/// What a placement's file records of it beyond where it stands and the shapes it covers.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelPlacementDetails {
  /// A planted particle system, by the effect or group it plays.
  Particles { effect: String },
  /// A static sound (`SStaticSound`): the sound, its volume and frequency, and its windows: the game day's hours it
  /// plays in, how long each play lasts and the pause between.
  Sound {
    sound: String,
    volume: f32,
    frequency: f32,
    active: LevelSoundWindow,
    play: LevelSoundWindow,
    pause: LevelSoundWindow,
  },
  /// A reverb environment's box: the environment outside it and inside it, by name.
  SoundEnvironment { outer: String, inner: String },
  /// A volumetric fog body: the profile under `$game_config$` it is drawn by, and how many obstacles cut it.
  FogVolume { profile: String, obstacles: u32 },
  /// An environment modifier (`CEnvModifier`): how strongly it pulls the weather at its centre, fading to none at its
  /// radius, toward its far plane, fog colour and density, ambient, sky and hemisphere colours, and which of those it
  /// moves, by name.
  EnvModifier {
    power: f32,
    far_plane: f32,
    fog_color: [f32; 3],
    fog_density: f32,
    ambient: [f32; 3],
    sky_color: [f32; 3],
    hemi_color: [f32; 3],
    affects: Vec<String>,
  },
  /// A multiplayer respawn point: what it respawns, its team, the game types it serves as a bit mask, and the item
  /// preset it spawns, empty for any other.
  RespawnPoint {
    role: String,
    team: u8,
    game_types: u16,
    profile: String,
  },
}
