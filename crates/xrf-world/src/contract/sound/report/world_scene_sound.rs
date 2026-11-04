use xrf_renderer::RenderSceneId;

use crate::contract::sound::report::world_sound_report::WorldSoundReport;

/// What the scene a viewport shows plays as the world schedules it, and the scene, which the host's audio is asked of
/// by its id.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldSceneSound {
  pub scene: RenderSceneId,
  pub report: WorldSoundReport,
}
