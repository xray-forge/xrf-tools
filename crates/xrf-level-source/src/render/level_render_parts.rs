use xrf_renderer::{LevelPart, LevelParts};
use xrf_session::{SessionId, SessionPart};

use crate::parts::{
  LevelAiMapPart, LevelCollisionPart, LevelDetailsPart, LevelEnvironmentPart, LevelGeometryPart, LevelGraphPart,
  LevelHemiPart, LevelLightsPart, LevelObjectMotionsPart, LevelParticlesPart, LevelPlacementsPart, LevelSectionsPart,
  LevelSpawnPart, LevelSpawnVisualsPart, LevelSurfacesPart, LevelWeatherPart,
};
use crate::sound::LevelSoundFilesPart;

/// How a level's session parts feed the scene the renderer draws it as: the scene each session shares, the parts whose
/// going stale streams it afresh, and the renderer's parts read again for the rest.
pub struct LevelRenderParts;

impl LevelRenderParts {
  /// The parts a level's scene is built from, whose going stale streams it afresh rather than reloading packages
  /// (`SessionChange::is_view_renewed`): its sectors are packed from its geometry and drawn with its surfaces, which no
  /// package reload replaces. Textures alone keep the scene, a file come or gone being loaded afresh by the texture
  /// watcher.
  pub const VIEW_PARTS: &'static [&'static str] = &[LevelGeometryPart::NAME, LevelSurfacesPart::NAME];

  /// What every source of a session's level shares its scene by, whichever of its snapshots it draws.
  pub fn format_scene_key(session_id: SessionId) -> String {
    format!("level {session_id}")
  }

  /// The renderer parts to reload when `stale_parts` went stale: each one read from any of them. Sectors stream afresh
  /// instead (`VIEW_PARTS`); the map and texture list are the page's, and the start matters only on first show.
  pub fn select_reload(stale_parts: &[&str]) -> LevelParts {
    LevelPart::ALL
      .into_iter()
      .filter(|part| {
        Self::list_read_from(*part)
          .iter()
          .any(|read| stale_parts.contains(read))
      })
      .collect()
  }

  /// The session parts `LevelRenderSource` reads `part` from; sectors have none (`VIEW_PARTS`). The spawn reads the
  /// sections for which artefacts a detector must reveal; the particles read the spawned visuals for the root bone an
  /// artefact plays its own at. Helpers include the sound emitters, whose files' distances size their spheres; weather
  /// reads its rain, thunder and ambient sounds.
  fn list_read_from(part: LevelPart) -> &'static [&'static str] {
    match part {
      LevelPart::Spawn => &[
        LevelSpawnPart::NAME,
        LevelSpawnVisualsPart::NAME,
        LevelSectionsPart::NAME,
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
        LevelSpawnVisualsPart::NAME,
        LevelSectionsPart::NAME,
        LevelEnvironmentPart::NAME,
        LevelCollisionPart::NAME,
        LevelHemiPart::NAME,
        LevelParticlesPart::NAME,
      ],
      LevelPart::Weather => &[
        LevelEnvironmentPart::NAME,
        LevelWeatherPart::NAME,
        LevelSoundFilesPart::NAME,
      ],
      LevelPart::Motions => &[LevelObjectMotionsPart::NAME],
      LevelPart::Grass => &[LevelDetailsPart::NAME],
      LevelPart::AiMap => &[LevelAiMapPart::NAME],
      LevelPart::Helpers => &[
        LevelSpawnPart::NAME,
        LevelGraphPart::NAME,
        LevelPlacementsPart::NAME,
        LevelSectionsPart::NAME,
        LevelSoundFilesPart::NAME,
      ],
      LevelPart::Sectors => &[],
    }
  }
}
