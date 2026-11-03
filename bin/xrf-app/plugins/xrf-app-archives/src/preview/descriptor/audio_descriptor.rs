use serde::Serialize;
use xrf_sound::{SoundFile, SoundMetadata};

use crate::preview::descriptor::AudioSourceParameters;

/// What a sound is, once it has been located.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDescriptor {
  /// Absent when the bytes carry no readable stream header.
  pub channels: Option<u16>,
  /// Absent when the bytes carry no readable stream header.
  pub sample_rate: Option<u32>,
  /// Absent for a sound carrying no recognized X-Ray comment, where the engine uses its own defaults.
  pub parameters: Option<AudioSourceParameters>,
}

impl AudioDescriptor {
  /// What the engine would read out of a sound's bytes.
  pub fn of(name: &str, bytes: &[u8]) -> Self {
    // A sound that cannot be parsed is still worth playing: plenty of ogg in a mod was not produced by the x-ray tools,
    // and refusing to describe it would take its preview down with the description.
    let sound: Option<SoundFile> = match SoundFile::read_from_bytes(bytes) {
      Ok(sound) => Some(sound),
      Err(error) => {
        log::warn!("Sound '{name}' carries no readable x-ray headers: {error}");

        None
      }
    };

    Self {
      channels: sound.as_ref().map(|it| it.channels),
      sample_rate: sound.as_ref().map(|it| it.sample_rate),
      parameters: sound.as_ref().and_then(|it| match &it.metadata {
        SoundMetadata::XRay { parameters, .. } => Some(AudioSourceParameters::from(parameters)),
        SoundMetadata::EngineDefaults => None,
      }),
    }
  }
}
