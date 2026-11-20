use std::path::PathBuf;

use tauri::State;
use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::window::InvokingWindow;
use xrf_app_render::{Renderer, WorldState};
use xrf_audio::AudioServer;
use xrf_renderer::{RenderFrameReport, RenderViewportId};

use crate::render::ShownLevels;
use crate::viewer::LevelViewerFolder;
use crate::viewer::recording::writer::MediaProbe;
use crate::viewer::recording::{LevelRecordings, RecordingProgress};

/// Record a viewport and what its camera hears into a new video in the recordings folder, at its current drawn size;
/// answers the file.
///
/// Until it stops, the recorded camera is heard whichever window has focus. `progress` gets a few updates a second,
/// and the reason when the recording ends by itself. The recording stops and finishes when the asking window's page
/// reloads or the window closes.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "start_recording")]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri injects each managed state as an argument of its own"
)]
pub(crate) async fn levels_start_recording(
  window: InvokingWindow,
  renderer: State<'_, Renderer>,
  shown: State<'_, ShownLevels>,
  world: State<'_, WorldState>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  recordings: State<'_, LevelRecordings>,
  viewport: RenderViewportId,
  name: String,
  progress: Channel<RecordingProgress>,
) -> TauriResult<PathBuf> {
  // Refused before the frame is asked for; the start itself checks again under the lock.
  if recordings.is_recording() {
    return Err(String::from("A recording runs already"));
  }

  MediaProbe::MEDIA_FOUNDATION.load()?;

  let server: AudioServer = shown.get_audio().server.clone();
  let frame: RenderFrameReport = renderer
    .describe_frame(viewport)
    .await?
    .filter(|frame| frame.width > 1 && frame.height > 1)
    .ok_or_else(|| format!("Viewport {} has drawn no frame to record", viewport.0))?;
  let folder: PathBuf = LevelViewerFolder::Recordings.get_path(&directories)?;
  let renderer: Renderer = renderer.inner().clone();
  let recordings: LevelRecordings = recordings.inner().clone();
  let world: WorldState = world.inner().clone();
  let window: String = window.get_label().to_owned();

  #[cfg(windows)]
  {
    use crate::viewer::recording::ViewportRecorder;

    execution
      .run_blocking("Starting a recording", move || -> TauriResult<PathBuf> {
        // Held from the check until the recorder is in place, so two concurrent starts never both pass.
        let mut active = recordings.lock();

        if active.as_ref().is_some_and(|recorder| !recorder.is_ended()) {
          return Err(String::from("A recording runs already"));
        }

        // Collect a recording that ended by itself (its file already finished) before the next opens its tap.
        if let Some(ended) = active.take() {
          let _ = ended.stop();
        }

        let file: PathBuf = LevelViewerFolder::find_free_recording(&LevelViewerFolder::make(folder)?, &name)?;

        *active = Some(ViewportRecorder::start(
          &renderer,
          &server,
          viewport,
          window,
          (frame.width, frame.height),
          file.clone(),
          progress,
        )?);
        // Pinned under the slot's lock, so a waiting stop unpins after, never before.
        world.lock().pin_listener(Some(viewport));
        drop(active);

        Ok(file)
      })
      .await?
  }

  #[cfg(not(windows))]
  {
    let _ = (
      frame, folder, renderer, recordings, world, execution, server, name, progress, window,
    );

    Err(MediaProbe::MEDIA_FOUNDATION.load().err().unwrap_or_default())
  }
}
