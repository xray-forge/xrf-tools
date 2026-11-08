//! What the renderer reads again of a shown level whose next snapshot let parts go, and what its scene is shared by.

use xrf_app_core::session::{SessionId, SessionPart};
use xrf_renderer::{LevelPart, LevelParts};

use crate::parts::{
  LevelCollisionPart, LevelDetailsPart, LevelEnvironmentPart, LevelGraphPart, LevelHemiPart, LevelLightsPart,
  LevelMapPart, LevelObjectMotionsPart, LevelParticlesPart, LevelPlacementsPart, LevelSectionsPart, LevelSpawnPart,
  LevelSpawnVisualsPart, LevelStartPart, LevelTexturesPart, LevelWeatherPart,
};
use crate::render::LevelRenderParts;

/// The set of `parts`.
fn parts(parts: &[LevelPart]) -> LevelParts {
  parts.iter().copied().collect()
}

#[test]
fn a_commit_leaving_every_part_reloads_nothing() {
  assert!(LevelRenderParts::select_reload(&[]).is_empty());
  assert!(
    LevelRenderParts::select_reload(&[LevelStartPart::NAME]).is_empty(),
    "where a viewer first stands draws nothing"
  );
  assert!(
    LevelRenderParts::select_reload(&[LevelMapPart::NAME]).is_empty(),
    "the map is the page's"
  );
  assert!(
    LevelRenderParts::select_reload(&[LevelTexturesPart::NAME]).is_empty(),
    "a texture come or gone is loaded afresh by the texture watcher"
  );
}

#[test]
fn what_a_render_read_recorded_reloads_that_package_alone() {
  for (part, reload) in [
    (LevelLightsPart::NAME, LevelPart::Lights),
    (LevelParticlesPart::NAME, LevelPart::Particles),
    (LevelWeatherPart::NAME, LevelPart::Weather),
    (LevelObjectMotionsPart::NAME, LevelPart::Motions),
    (LevelDetailsPart::NAME, LevelPart::Grass),
    // The level objects, which the helpers draw and the level's sounds are read from.
    (LevelGraphPart::NAME, LevelPart::Helpers),
    (LevelPlacementsPart::NAME, LevelPart::Helpers),
  ] {
    assert_eq!(LevelRenderParts::select_reload(&[part]), parts(&[reload]), "{part}");
  }
}

#[test]
fn a_stale_part_reloads_only_the_packages_built_from_it() {
  assert_eq!(
    LevelRenderParts::select_reload(&[LevelEnvironmentPart::NAME]),
    parts(&[LevelPart::Particles, LevelPart::Weather])
  );
  assert_eq!(
    LevelRenderParts::select_reload(&[LevelSectionsPart::NAME]),
    parts(&[LevelPart::Lights, LevelPart::Helpers, LevelPart::Particles])
  );
  assert_eq!(
    LevelRenderParts::select_reload(&[LevelSpawnVisualsPart::NAME]),
    parts(&[LevelPart::Spawn, LevelPart::Lights])
  );
  assert_eq!(
    LevelRenderParts::select_reload(&[LevelCollisionPart::NAME, LevelHemiPart::NAME]),
    parts(&[LevelPart::Spawn, LevelPart::Particles])
  );
  assert_eq!(
    LevelRenderParts::select_reload(&[LevelSpawnPart::NAME]),
    parts(&[
      LevelPart::Spawn,
      LevelPart::Lights,
      LevelPart::Helpers,
      LevelPart::Particles
    ])
  );
}

#[test]
fn a_levels_scene_is_named_by_its_session_alone() {
  let (session, other) = (SessionId::generate(), SessionId::generate());

  assert_eq!(
    LevelRenderParts::format_scene_key(session),
    LevelRenderParts::format_scene_key(session)
  );
  assert_ne!(
    LevelRenderParts::format_scene_key(session),
    LevelRenderParts::format_scene_key(other)
  );
}
