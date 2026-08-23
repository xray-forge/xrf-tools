use serde::Serialize;
use tauri::State;
use xrf_sound::{SoundFile, SoundMetadata};
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::types::TauriResult;

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
///
/// Every field is optional because there are two independent ways to know less than everything: bytes that are not a
/// readable ogg at all, and a perfectly good ogg carrying no X-Ray comment. Reporting a zero for either would make an
/// unreadable file indistinguishable from a silent one.
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

/// Report whatever the engine would read out of a sound, without handing over the sound.
///
/// Paired with `assets|read_asset`, which serves the bytes the webview plays. Both are addressed by the same roots and
/// logical path, so the numbers on screen describe the file that is playing rather than a second lookup's answer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_audio"))]
#[tauri::command(rename = "describe_audio")]
pub async fn archives_describe_audio(
  roots: XrayRoots,
  logical_path: String,
  assets: State<'_, AssetMountState>,
) -> TauriResult<AudioDescriptor> {
  log::info!("Describing audio: {logical_path}");

  let bytes: Vec<u8> = assets
    .with_probe(&roots, |probe| read_located_asset(probe, &logical_path))?
    .map_err(|error| format!("Failed to describe audio '{logical_path}': {error}"))?;

  // A sound that cannot be parsed is still worth playing: plenty of ogg in a mod was not produced by the x-ray tools,
  // and refusing to describe it would take the playable bytes down with the description.
  let sound: Option<SoundFile> = match SoundFile::read_from_bytes(&bytes) {
    Ok(sound) => Some(sound),
    Err(error) => {
      log::warn!("Sound '{logical_path}' carries no readable x-ray headers: {error}");

      None
    }
  };

  Ok(AudioDescriptor {
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
  })
}
