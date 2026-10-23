//! What the renderer reads again of a shown level whose next snapshot let parts go, and what its scene is shared by.

use xrf_world::WorldLevelReload;

use crate::core::session::SessionId;
use crate::plugins::levels::collision::LEVEL_COLLISION_PART;
use crate::plugins::levels::configs::LEVEL_SECTIONS_PART;
use crate::plugins::levels::details::LEVEL_DETAILS_PART;
use crate::plugins::levels::hemi::LEVEL_HEMI_PART;
use crate::plugins::levels::lights::LEVEL_LIGHTS_PART;
use crate::plugins::levels::object_motions::LEVEL_OBJECT_MOTIONS_PART;
use crate::plugins::levels::particles::LEVEL_PARTICLES_PART;
use crate::plugins::levels::render_source::LevelRenderSource;
use crate::plugins::levels::spawn::{LEVEL_GRAPH_PART, LEVEL_SPAWN_PART};
use crate::plugins::levels::spawn_visuals::LEVEL_SPAWN_VISUALS_PART;
use crate::plugins::levels::start::LEVEL_START_PART;
use crate::plugins::levels::state::level_environment::{LEVEL_ENVIRONMENT_PART, LEVEL_WEATHER_PART};

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
    LevelRenderSource::to_reload(&[LEVEL_GRAPH_PART]).is_empty(),
    "nothing drawn is built from the graph"
  );
  assert!(
    LevelRenderSource::to_reload(&[LEVEL_START_PART]).is_empty(),
    "where a viewer first stands draws nothing"
  );
}

#[test]
fn what_a_render_read_recorded_reloads_that_package_alone() {
  for (part, reload) in [
    (
      LEVEL_LIGHTS_PART,
      WorldLevelReload {
        is_lights: true,
        ..NONE
      },
    ),
    (
      LEVEL_PARTICLES_PART,
      WorldLevelReload {
        is_particles: true,
        ..NONE
      },
    ),
    (
      LEVEL_WEATHER_PART,
      WorldLevelReload {
        is_weather: true,
        ..NONE
      },
    ),
    (
      LEVEL_OBJECT_MOTIONS_PART,
      WorldLevelReload {
        is_motions: true,
        ..NONE
      },
    ),
    (LEVEL_DETAILS_PART, WorldLevelReload { is_grass: true, ..NONE }),
  ] {
    assert_eq!(LevelRenderSource::to_reload(&[part]), reload, "{part}");
  }
}

#[test]
fn a_stale_part_reloads_only_the_packages_built_from_it() {
  assert_eq!(
    LevelRenderSource::to_reload(&[LEVEL_ENVIRONMENT_PART]),
    WorldLevelReload {
      is_particles: true,
      is_weather: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LEVEL_SECTIONS_PART]),
    WorldLevelReload {
      is_lights: true,
      is_particles: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LEVEL_SPAWN_VISUALS_PART]),
    WorldLevelReload {
      is_spawn: true,
      is_lights: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LEVEL_COLLISION_PART, LEVEL_HEMI_PART]),
    WorldLevelReload {
      is_spawn: true,
      is_particles: true,
      ..NONE
    }
  );
  assert_eq!(
    LevelRenderSource::to_reload(&[LEVEL_SPAWN_PART]),
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
  let (session, other) = (SessionId::new(), SessionId::new());

  assert_eq!(
    LevelRenderSource::to_scene_key(session),
    LevelRenderSource::to_scene_key(session)
  );
  assert_ne!(
    LevelRenderSource::to_scene_key(session),
    LevelRenderSource::to_scene_key(other)
  );
}
