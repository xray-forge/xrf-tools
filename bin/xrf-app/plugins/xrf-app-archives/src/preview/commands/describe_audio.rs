use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_extension::XrayExtensionOf;
use xrf_sound::{SoundFile, SoundMetadata};

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// The X-Ray source parameters carried in a sound's first vorbis comment.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceParameters {
  pub min_distance: f32,
  pub max_distance: f32,
  pub base_volume: f32,
  pub game_type: u32,
  pub max_ai_distance: f32,
}

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
  /// What to hand the bytes over as, taken from the extension rather than from the content.
  pub media_type: String,
}

impl AudioDescriptor {
  /// What the engine would read out of a sound's bytes, handed over as `media_type`.
  fn of(name: &str, bytes: &[u8], media_type: &str) -> Self {
    // A sound that cannot be parsed is still worth playing: plenty of ogg in a mod was not produced by the x-ray tools,
    // and refusing to describe it would take the playable bytes down with the description.
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
        SoundMetadata::XRay { parameters, .. } => Some(AudioSourceParameters {
          min_distance: parameters.min_distance,
          max_distance: parameters.max_distance,
          base_volume: parameters.base_volume,
          game_type: parameters.game_type,
          max_ai_distance: parameters.max_ai_distance,
        }),
        SoundMetadata::EngineDefaults => None,
      }),
      media_type: media_type.to_owned(),
    }
  }
}

/// Report whatever the engine would read out of a sound of the open subject, without handing over the sound.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_audio"))]
#[tauri::command(rename = "describe_audio")]
pub async fn archives_describe_audio(
  session_id: SessionId,
  name: String,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<AudioDescriptor> {
  log::info!("Describing audio: {name}");

  let media_type: &'static str = XrayExtensionOf::of(&name)
    .known()
    .and_then(|extension| extension.get_media_type())
    .ok_or_else(|| format!("Failed to describe audio '{name}': no webview plays this extension"))?;
  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  execution
    .run_blocking("Describing audio", move || {
      Ok::<_, String>(AudioDescriptor::of(&name, &subject.read_bytes(&name)?, media_type))
    })
    .await?
}
