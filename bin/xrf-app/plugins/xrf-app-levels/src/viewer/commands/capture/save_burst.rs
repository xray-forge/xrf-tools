use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::{Renderer, WorldState};
use xrf_level_source::LevelAudio;
use xrf_renderer::BurstEvent;

use crate::render::ShownLevels;
use crate::viewer::LevelViewerFolder;
use crate::viewer::burst::{BurstCaptureRequest, BurstOutcome, BurstSound, BurstVideo, BurstWriter};
use crate::viewer::recording::writer::MediaProbe;

/// Draw a burst of consecutive frames in a viewport and write each with its G-buffer marks into a new bursts folder,
/// plus the camera's sound and a video when asked; answers once the last is written.
///
/// A burst with sound is refused up front when the viewport's level plays none; a burst that fails once its folder is
/// made removes the folder.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_burst")]
pub(crate) async fn levels_save_burst(
  renderer: State<'_, Renderer>,
  world: State<'_, WorldState>,
  shown: State<'_, ShownLevels>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  request: BurstCaptureRequest,
) -> TauriResult<BurstOutcome> {
  let BurstCaptureRequest {
    viewport,
    burst,
    name,
    has_sound,
    has_video,
  } = request;

  if has_sound && !world.lock().has_sound(viewport) {
    return Err(String::from(
      "The viewport's level plays no sound, so a burst has none to record",
    ));
  }

  let bursts: PathBuf = LevelViewerFolder::Bursts.get_path(&directories)?;
  let renderer: Renderer = renderer.inner().clone();
  let audio: LevelAudio = shown.get_audio().clone();

  execution
    .run_blocking("Saving a burst", move || -> TauriResult<BurstOutcome> {
      let folder: PathBuf = LevelViewerFolder::find_new_burst(&bursts, &name)?;
      // Refused before anything draws when this system cannot write video.
      let mut video: Option<BurstVideo> = has_video
        .then(|| BurstVideo::new(&burst, &MediaProbe::MEDIA_FOUNDATION, &folder))
        .transpose()?;
      // The sound needs the burst's first frame, so it starts before the burst.
      let sound: Option<BurstSound> = has_sound
        .then(|| {
          BurstSound::start(
            &burst,
            &audio.server,
            &audio.bursts,
            video.as_mut().and_then(BurstVideo::take_sound),
          )
        })
        .transpose()?;
      let mut writer: BurstWriter = BurstWriter::new(LevelViewerFolder::make(folder)?, &burst);

      if let Some(sound) = sound {
        writer = writer.with_sound(sound);
      }

      if let Some(video) = video {
        writer = writer.with_video(video);
      }

      let events: Receiver<BurstEvent> = renderer.capture_burst(viewport, burst);

      writer.receive(&events)
    })
    .await?
}
