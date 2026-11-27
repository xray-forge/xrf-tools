use std::sync::Arc;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_movement::MovementCamera;
use xrf_level_source::{LevelSource, LevelStart, LevelStartPart, OpenedLevel, SelectedLevel};
use xrf_math::glam::Vec3;
use xrf_renderer::EngineSpace;
use xrf_session::{SessionDerived, SessionId, SessionSnapshot};
use xrf_vfs::{XrayRevision, XraySearch};

use crate::level::game_tree::GameTree;

/// A level of the game tree, opened: what the world shows and where the player starts on it.
pub struct GameLevel {
  snapshot: Arc<SessionSnapshot<SelectedLevel>>,
  start: Option<LevelStart>,
}

impl GameLevel {
  /// Metres above the level's origin the camera stands where the level names no start.
  const FALLBACK_HEIGHT: f32 = 2.0;

  /// Opens the level `name` of `tree`, reading on `pool` as its reads fan out.
  ///
  /// # Errors
  ///
  /// Returns why the level cannot be read: the tree names no such level, or its files do not read.
  pub fn open(tree: &GameTree, name: &str, pool: &ThreadPool) -> XrfResult<Self> {
    let levels: Vec<String> = tree.list_levels()?;

    if !levels.iter().any(|level| level == name) {
      return Err(XrfError::new_not_found_error(format!(
        "the tree has no level '{name}': it has {}",
        levels.join(", ")
      )));
    }

    let search: XraySearch = tree.search()?;
    let revision: XrayRevision = search.get_vfs().get_revision();
    let source: LevelSource = LevelSource::Asset {
      logical_path: format!("{}\\{name}", LevelSource::LEVELS_DIRECTORY),
    };
    let engine = tree.get_engine().clone();
    let derived: SessionDerived<SelectedLevel> = pool.install(|| {
      OpenedLevel::read(
        search.clone(),
        source,
        tree.get_roots().clone(),
        tree.get_dialect(),
        |_, _| engine,
      )
    })?;
    let snapshot: Arc<SessionSnapshot<SelectedLevel>> = Arc::new(SessionSnapshot {
      revision,
      inputs: derived.inputs,
      parts: derived.parts,
      ..SessionSnapshot::new(SessionId::generate(), derived.value)
    });
    let start: Option<LevelStart> = LevelStartPart::get(&snapshot, &search);

    Ok(Self { snapshot, start })
  }

  pub fn get_snapshot(&self) -> &Arc<SessionSnapshot<SelectedLevel>> {
    &self.snapshot
  }

  /// Where the player's camera starts, in the engine's space: the level's start (an arrival or the actor's eye under
  /// open sky), else above its origin looking along `+z`.
  pub fn get_start_camera(&self) -> MovementCamera {
    let Some(start) = &self.start else {
      return MovementCamera::looking_at(
        Vec3::Y * Self::FALLBACK_HEIGHT,
        Vec3::new(0.0, Self::FALLBACK_HEIGHT, 1.0),
      );
    };
    // The start is in the renderer's space, which differs from the engine's only in `z`.
    let position: Vec3 = EngineSpace::mirror(Vec3::new(start.position.x, start.position.y, start.position.z));
    let direction: Vec3 = EngineSpace::mirror(Vec3::new(start.direction.x, start.direction.y, start.direction.z));

    MovementCamera::looking_at(position, position + direction)
  }
}
