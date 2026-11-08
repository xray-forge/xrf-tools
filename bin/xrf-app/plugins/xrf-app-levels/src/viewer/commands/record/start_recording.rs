use std::path::PathBuf;

use tauri::State;
use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::{Renderer, WorldState};
use xrf_renderer::{RenderFrameReport, RenderViewportId};

use crate::render::ShownLevels;
use crate::viewer::LevelViewerFolder;
use crate::viewer::recording::writer::MediaProbe;
use crate::viewer::recording::{LevelRecordings, RecordingProgress};

/// Record a viewport with what its camera hears into a new video of the level viewer's recordings named as asked, at
/// the size it is drawn at now, its camera heard whatever window takes the focus until it stops; `progress` is told
/// where it stands a few times a second, and why where it ends by itself. Answers the file.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "start_recording")]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri injects each managed state as an argument of its own"
)]
pub async fn levels_start_recording(
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

  let server = shown
    .get_audio()
    .server
    .clone()
    .ok_or("The audio did not start, so a recording would have no sound")?;
  let frame: RenderFrameReport = renderer
    .describe_frame(viewport)
    .await?
    .filter(|frame| frame.width > 1 && frame.height > 1)
    .ok_or_else(|| format!("Viewport {} has drawn no frame to record", viewport.0))?;
  let folder: PathBuf = LevelViewerFolder::Recordings.get_path(&directories)?;
  let renderer: Renderer = renderer.inner().clone();
  let recordings: LevelRecordings = recordings.inner().clone();
  let world: WorldState = world.inner().clone();

  #[cfg(windows)]
  {
    use crate::viewer::recording::ViewportRecorder;

    execution
      .run_blocking("Starting a recording", move || -> TauriResult<PathBuf> {
        // Held from the check to the recorder taking its place, so two starts asked at once never both pass it.
        let mut active = recordings.lock();

        if active.as_ref().is_some_and(|recorder| !recorder.is_ended()) {
          return Err(String::from("A recording runs already"));
        }

        // One that ended by itself is collected, its file finished already, before the next opens its tap.
        if let Some(ended) = active.take() {
          let _ = ended.stop();
        }

        let file: PathBuf = LevelViewerFolder::find_free_recording(&LevelViewerFolder::make(folder)?, &name)?;

        *active = Some(ViewportRecorder::start(
          &renderer,
          &server,
          viewport,
          (frame.width, frame.height),
          file.clone(),
          progress,
        )?);
        drop(active);

        world.lock().pin_listener(Some(viewport));

        Ok(file)
      })
      .await?
  }

  #[cfg(not(windows))]
  {
    let _ = (
      frame, folder, renderer, recordings, world, execution, server, name, progress,
    );

    Err(MediaProbe::MEDIA_FOUNDATION.load().err().unwrap_or_default())
  }
}
