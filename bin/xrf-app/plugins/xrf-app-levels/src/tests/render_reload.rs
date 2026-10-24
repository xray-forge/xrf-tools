//! What the renderer reads again of a shown level whose next snapshot let parts go, and what its scene is shared by.

use xrf_app_core::session::{SessionId, SessionPart};
use xrf_world::WorldLevelReload;

use crate::parts::{
  LevelCollisionPart, LevelDetailsPart, LevelEnvironmentPart, LevelGraphPart, LevelHemiPart, LevelLightsPart,
  LevelMapPart, LevelObjectMotionsPart, LevelParticlesPart, LevelSectionsPart, LevelSpawnPart, LevelSpawnVisualsPart,
  LevelStartPart, LevelTexturesPart, LevelWeatherPart,
};
use crate::render::LevelRenderSource;

const NONE: WorldLevelReload = WorldLevelReload {
  is_spawn: false,
  is_lights: false,
  is_particles: false,
  is_weather: false,
  is_motions: false,
  is_grass: false,
};

#[test]
fn a_commit_leaving_every_part_reloads_nothing() {
  assert!(LevelRenderSource::to_reload(&[]).is_empty());
  assert!(
    LevelRenderSource::to_reload(&[LevelGraphPart::NAME]).is_empty(),
    "nothing drawn is built from the graph"
  );
  assert!(
    LevelRenderSource::to_reload(&[LevelStartPart::NAME]).is_empty(),
    "where a viewer first stands draws nothing"
  );
  assert!(
    LevelRenderSource::to_reload(&[LevelMapPart::NAME]).is_empty(),
    "the map is the page's"
  );
  assert!(
    LevelRenderSource::to_reload(&[LevelTexturesPart::NAME]).is_empty(),
    "a texture come or gone is loaded afresh by the texture watcher"
  );
}

#[test]
fn what_a_render_read_recorded_reloads_that_package_alone() {
  for (part, reload) in [
    (
      LevelLightsPart::NAME,
      WorldLevelReload {
        is_lights: true,
        ..NONE
      },
    ),
    (
      LevelParticlesPart::NAME,
      WorldLevelReload {
        is_particles: true,
        ..NONE
      },
    ),
    (
      LevelWeatherPart::NAME,
      WorldLevelReload {
        is_weather: true,
        ..NONE
      },
    ),
    (
      LevelObjectMotionsPart::NAME,
      WorldLevelReload {
        is_motions: true,
        ..NONE
      },
    ),
    (LevelDetailsPart::NAME, WorldLevelReload { is_grass: true, ..NONE }),
  ] {
    assert_eq!(LevelRenderSource::to_reload(&[part]), reload, "{part}");
  }
}

#[test]
fn a_stale_part_reloads_only_the_packages_built_from_it() {
  assert_eq!(
    LevelRenderSource::to_reload(&[LevelEnvironmentPart::NAME]),
    WorldLevelReload {
      is_particles: true,
      is_weather: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LevelSectionsPart::NAME]),
    WorldLevelReload {
      is_lights: true,
      is_particles: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LevelSpawnVisualsPart::NAME]),
    WorldLevelReload {
      is_spawn: true,
      is_lights: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LevelCollisionPart::NAME, LevelHemiPart::NAME]),
    WorldLevelReload {
      is_spawn: true,
      is_particles: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LevelSpawnPart::NAME]),
    WorldLevelReload {
      is_spawn: true,
      is_lights: true,
      is_particles: true,
      ..NONE
    }
  );
}

#[test]
fn a_levels_scene_is_named_by_its_session_alone() {
  let (session, other) = (SessionId::generate(), SessionId::generate());

  assert_eq!(
    LevelRenderSource::to_scene_key(session),
    LevelRenderSource::to_scene_key(session)
  );
  assert_ne!(
    LevelRenderSource::to_scene_key(session),
    LevelRenderSource::to_scene_key(other)
  );
}
