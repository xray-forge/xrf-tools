use xrf_app_core::session::{SessionId, SessionPart};
use xrf_renderer::{LevelPart, LevelParts};

use crate::parts::{
  LevelCollisionPart, LevelDetailsPart, LevelEnvironmentPart, LevelGeometryPart, LevelGraphPart, LevelHemiPart,
  LevelLightsPart, LevelObjectMotionsPart, LevelParticlesPart, LevelPlacementsPart, LevelSectionsPart, LevelSpawnPart,
  LevelSpawnVisualsPart, LevelSurfacesPart, LevelWeatherPart,
};

/// How a level's session parts feed the scene the renderer draws it as: the scene each session shares, the parts whose
/// going stale streams it afresh, and the renderer's parts read again for the rest.
pub(crate) struct LevelRenderParts;

impl LevelRenderParts {
  /// The parts a level's scene is built from, whose going stale streams it afresh rather than reloading packages
  /// (`SessionChange::is_view_renewed`): its sectors are packed from its geometry and drawn with its surfaces, which no
  /// package reload replaces. Textures alone keep the scene, a file come or gone being loaded afresh by the texture
  /// watcher.
  pub(crate) const VIEW_PARTS: &'static [&'static str] = &[LevelGeometryPart::NAME, LevelSurfacesPart::NAME];

  /// What every source of a session's level shares its scene by, whichever of its snapshots it draws.
  pub(crate) fn format_scene_key(session_id: SessionId) -> String {
    format!("level {session_id}")
  }

  /// What the renderer reads again of a level whose next snapshot let `stale_parts` go: each package whose read asks
  /// for any of them, or recorded what it read as one of them. The sectors stream afresh instead, on their geometry or
  /// surfaces (`VIEW_PARTS`); the map and the textures' list are the page's, and the start only matters where a viewer
  /// first stands.
  pub(crate) fn select_reload(stale_parts: &[&str]) -> LevelParts {
    LevelPart::ALL
      .into_iter()
      .filter(|part| {
        Self::list_read_from(*part)
          .iter()
          .any(|read| stale_parts.contains(read))
      })
      .collect()
  }

  /// The session parts the renderer's read of `part` is built from, as `LevelRenderSource` reads it. The sectors
  /// stream afresh on theirs instead (`VIEW_PARTS`). The helpers are the level objects, which the level's sounds are
  /// read from too: the spawn, the graph, the placements (`level.snd_static` among them) and the sections.
  fn list_read_from(part: LevelPart) -> &'static [&'static str] {
    match part {
      LevelPart::Spawn => &[
        LevelSpawnPart::NAME,
        LevelSpawnVisualsPart::NAME,
        LevelCollisionPart::NAME,
        LevelHemiPart::NAME,
      ],
      LevelPart::Lights => &[
        LevelSpawnPart::NAME,
        LevelSpawnVisualsPart::NAME,
        LevelSectionsPart::NAME,
        LevelLightsPart::NAME,
      ],
      LevelPart::Particles => &[
        LevelSpawnPart::NAME,
        LevelSectionsPart::NAME,
        LevelEnvironmentPart::NAME,
        LevelCollisionPart::NAME,
        LevelHemiPart::NAME,
        LevelParticlesPart::NAME,
      ],
      LevelPart::Weather => &[LevelEnvironmentPart::NAME, LevelWeatherPart::NAME],
      LevelPart::Motions => &[LevelObjectMotionsPart::NAME],
      LevelPart::Grass => &[LevelDetailsPart::NAME],
      LevelPart::Helpers => &[
        LevelSpawnPart::NAME,
        LevelGraphPart::NAME,
        LevelPlacementsPart::NAME,
        LevelSectionsPart::NAME,
      ],
      LevelPart::Sectors => &[],
    }
  }
}
