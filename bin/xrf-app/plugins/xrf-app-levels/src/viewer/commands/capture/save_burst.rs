use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::Renderer;
use xrf_renderer::BurstEvent;

use crate::render::{LevelAudio, ShownLevels};
use crate::viewer::LevelViewerFolder;
use crate::viewer::burst::{BurstCaptureRequest, BurstOutcome, BurstSound, BurstWriter};

/// Draw a burst of consecutive frames in a viewport and write each, with its G-buffer's marks, into a new folder of
/// the level viewer's bursts named as asked, and the sound its camera hears at the burst's times beside them where
/// asked; answers once the last is written.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_burst")]
pub async fn levels_save_burst(
  renderer: State<'_, Renderer>,
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
  } = request;
  let bursts: PathBuf = LevelViewerFolder::Bursts.get_path(&directories)?;
  let renderer: Renderer = renderer.inner().clone();
  let audio: LevelAudio = shown.get_audio().clone();

  if has_sound && audio.server.is_none() {
    return Err(String::from(
      "The audio did not start, so a burst has no sound to record",
    ));
  }

  execution
    .run_blocking("Saving a burst", move || -> TauriResult<BurstOutcome> {
      let folder: PathBuf = LevelViewerFolder::find_new_burst(&bursts, &name)?;
      // The sound takes the burst's frames from its first, so it starts before the burst does.
      let sound: Option<BurstSound> = match audio.server.as_ref().filter(|_| has_sound) {
        Some(server) => Some(BurstSound::start(&burst, server, &audio.bursts)?),
        None => None,
      };
      let mut writer: BurstWriter = BurstWriter::new(LevelViewerFolder::make(folder)?, &burst);

      if let Some(sound) = sound {
        writer = writer.with_sound(sound);
      }

      let events: Receiver<BurstEvent> = renderer.capture_burst(viewport, burst);

      writer.receive(&events)
    })
    .await?
}
