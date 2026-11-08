use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::WorldState;
use xrf_audio::{AudioSceneReport, AudioServer};
use xrf_renderer::{RenderSceneId, RenderViewportId};
use xrf_world::{WorldSceneSound, WorldSoundReport};

use crate::sound::LevelSoundReport;

/// What the level a viewport shows plays and why: each sound, where it stands and how loud it is heard, the music, the
/// ambient's channels, and every sound that could not play; none for a viewport showing no level. The world says what
/// it schedules, the audio how each is heard, asked on a blocking thread since it answers from its own.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_sound")]
pub(crate) async fn levels_describe_sound(
  world: State<'_, WorldState>,
  audio: State<'_, AudioServer>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<Option<WorldSoundReport>> {
  let Some(WorldSceneSound {
    scene: RenderSceneId(scene),
    report,
  }) = world.lock().report_sound(viewport)
  else {
    return Ok(None);
  };
  let audio: AudioServer = AudioServer::clone(&audio);
  let heard: Option<AudioSceneReport> = execution
    .run_blocking("Describing the level's sound", move || {
      audio.describe_scene(scene).map_err(|error| error.to_string())
    })
    .await??;

  Ok(Some(match heard {
    Some(heard) => LevelSoundReport::join(report, heard),
    None => report,
  }))
}
