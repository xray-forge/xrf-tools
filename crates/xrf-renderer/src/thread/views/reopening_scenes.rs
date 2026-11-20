use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::host::world::render_scene_id::RenderSceneId;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::texture::texture_cache::TextureCache;

/// Scenes shown when the GPU was released, each held back once remade until it is whole, so a restart shows the level
/// at once rather than streaming in.
///
/// A scene is held at most `WAIT_MOST` from its first remade frame; the GPU's start and retries do not count.
#[derive(Default)]
pub(crate) struct ReopeningScenes {
  /// Each held scene, with when it was first asked for once remade; none until then.
  held: HashMap<RenderSceneId, Option<Instant>>,
}

impl ReopeningScenes {
  /// The longest a remade scene is held back waiting to be whole.
  pub(crate) const WAIT_MOST: Duration = Duration::from_secs(10);

  /// Holds `scenes` back after their GPU was released.
  pub(crate) fn hold(&mut self, scenes: impl IntoIterator<Item = RenderSceneId>) {
    self.held.extend(scenes.into_iter().map(|scene| (scene, None)));
  }

  pub(crate) fn is_held(&self, scene: RenderSceneId) -> bool {
    self.held.contains_key(&scene)
  }

  /// Whether a held scene, remade as `level`, may be shown: once whole or after `WAIT_MOST`. Releasing it logs the wait
  /// and, if not whole, what it still lacks.
  pub(crate) fn release(&mut self, scene: RenderSceneId, level: &LevelScene, textures: &TextureCache) -> bool {
    let Some(asked) = self.held.get_mut(&scene) else {
      return true;
    };
    let waited: Duration = asked.get_or_insert_with(Instant::now).elapsed();
    let is_ready: bool = level.is_ready(textures);

    if !is_ready && waited < Self::WAIT_MOST {
      return false;
    }

    self.held.remove(&scene);

    match is_ready {
      true => log::info!(
        "Level shown again {} ms after its scene was made again",
        waited.as_millis()
      ),
      false => log::warn!(
        "Level shown again {} ms after its scene was made again, before it is whole: {}",
        waited.as_millis(),
        level.describe_pending(textures)
      ),
    }

    true
  }

  /// Lets go of the scenes the world no longer streams.
  pub(crate) fn retain(&mut self, live: &HashSet<RenderSceneId>) {
    self.held.retain(|scene, _| live.contains(scene));
  }
}
