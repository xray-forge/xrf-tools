use xrf_renderer::RenderSceneId;

use crate::contract::sound::report::world_sound_report::WorldSoundReport;

/// The sound report the world schedules for a viewport's scene, keyed by the scene's id for the host's audio.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldSceneSound {
  pub scene: RenderSceneId,
  pub report: WorldSoundReport,
}
