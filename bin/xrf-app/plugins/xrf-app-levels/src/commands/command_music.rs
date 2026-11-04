use tauri::State;
use xrf_app_render::WorldState;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldMusicCommand;

/// Plays the music of the level a viewport shows as asked: by the game's schedule, one track by hand, or the track
/// playing from a point; answers whether it could, which it cannot for a viewport showing no level, a track the level
/// does not have, or a seek between tracks.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "command_music")]
pub fn levels_command_music(
  world: State<'_, WorldState>,
  viewport: RenderViewportId,
  command: WorldMusicCommand,
) -> bool {
  world.lock().command_music(viewport, &command)
}
