use tauri::State;
use xrf_app_render::WorldState;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldMusicCommand;

/// Control the music of the level a viewport shows: the game's schedule, one chosen track, or a seek within the playing
/// track.
///
/// Answers false for a viewport showing no level, a track the level lacks, or a seek across tracks.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "command_music")]
pub(crate) fn levels_command_music(
  world: State<'_, WorldState>,
  viewport: RenderViewportId,
  command: WorldMusicCommand,
) -> bool {
  world.lock().command_music(viewport, &command)
}
