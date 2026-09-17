use serde::Serialize;
use xrf_db::SoundEnvironment;

/// One reverb preset, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSoundEnvironment {
  /// The name a level's sound environments reach this preset by.
  pub name: String,
  pub version: u32,
  /// How long the reverb takes to fall away, in seconds.
  pub decay_time: f32,
  /// How much the room adds at low and at high frequencies, in hundredths of a decibel.
  pub room: f32,
  pub room_hf: f32,
  /// How big the space sounds, in metres.
  pub environment_size: f32,
  /// The EAX preset it stands for, which only version 4 and above declares.
  pub environment: Option<u32>,
}

impl ArchiveSoundEnvironment {
  /// Every preset of a library, in the order it numbers them, which is how a level addresses one.
  pub fn of_all(environments: &[SoundEnvironment]) -> Vec<Self> {
    environments.iter().map(Self::of).collect()
  }

  /// One preset, taken over what it does to what is heard in it.
  fn of(environment: &SoundEnvironment) -> Self {
    Self {
      name: environment.name.clone(),
      version: environment.version,
      decay_time: environment.decay_time,
      room: environment.room,
      room_hf: environment.room_hf,
      environment_size: environment.environment_size,
      environment: environment.environment,
    }
  }
}
