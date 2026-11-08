use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::host::world::render_scene_id::RenderSceneId;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::texture::texture_cache::TextureCache;

/// The scenes viewports showed when the GPU was let go, each held back once made again until it is whole, so a restart
/// shows the level at once rather than streaming in; held at most `WAIT_MOST` from when it was first asked for made
/// again, after which it is shown as it is. The GPU's start and its retries count none of that wait.
#[derive(Default)]
pub(crate) struct ReopeningScenes {
  /// Each held scene, by when it was first asked for once made again; none until it is.
  held: HashMap<RenderSceneId, Option<Instant>>,
}

impl ReopeningScenes {
  /// The longest a scene made again is held back for being whole.
  pub(crate) const WAIT_MOST: Duration = Duration::from_secs(10);

  /// Holds `scenes` back, the GPU they were shown on let go.
  pub(crate) fn hold(&mut self, scenes: impl IntoIterator<Item = RenderSceneId>) {
    self.held.extend(scenes.into_iter().map(|scene| (scene, None)));
  }

  pub(crate) fn is_held(&self, scene: RenderSceneId) -> bool {
    self.held.contains_key(&scene)
  }

  /// Whether a held scene, made again as `level`, may be shown now: once it is whole, or once it was held `WAIT_MOST`
  /// since it was first asked for made again, letting it go then and logging how long it was held and, where it is not
  /// whole, what it still waits for.
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
