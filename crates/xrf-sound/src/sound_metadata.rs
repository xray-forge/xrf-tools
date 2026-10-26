use serde::{Deserialize, Serialize};

use crate::vorbis::vorbis_comment::VorbisComment;
use crate::xray::xray_sound_comment_version::XRaySoundCommentVersion;
use crate::xray::xray_sound_parameters::XRaySoundParameters;

/// Metadata that controls how X-Ray configures a sound source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SoundMetadata {
  /// The engine uses its built-in sound-source defaults when no recognized X-Ray comment exists.
  EngineDefaults,
  XRay {
    version: XRaySoundCommentVersion,
    parameters: XRaySoundParameters,
  },
}

impl SoundMetadata {
  /// Reads the X-Ray parameters the first comment holds, as `CSoundRender_Source::LoadWave` does.
  pub(crate) fn read(comment: &VorbisComment) -> Result<Self, String> {
    let Some(first) = comment.get_first() else {
      return Ok(Self::EngineDefaults);
    };

    Ok(match XRaySoundParameters::read(first)? {
      Some((version, parameters)) => Self::XRay { version, parameters },
      None => Self::EngineDefaults,
    })
  }

  /// The parameters the engine plays the sound with: the ones it carries, or the engine's defaults.
  pub fn get_parameters(&self) -> XRaySoundParameters {
    match self {
      Self::EngineDefaults => XRaySoundParameters::default(),
      Self::XRay { parameters, .. } => parameters.clone(),
    }
  }
}
