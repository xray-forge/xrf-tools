use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::host::render_scene_id::RenderSceneId;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::texture::texture_cache::TextureCache;

/// The scenes viewports showed when the GPU was let go, each held back once made again until it is whole, so a restart
/// shows the level at once rather than streaming in; held at most `WAIT_MOST`, after which it is shown as it is.
#[derive(Default)]
pub struct ReopeningScenes {
  /// Each held scene, by when the GPU it was shown on was let go.
  held: HashMap<RenderSceneId, Instant>,
}

impl ReopeningScenes {
  /// The longest a scene is held back for being whole.
  pub const WAIT_MOST: Duration = Duration::from_secs(10);

  /// Holds `scenes` back from now, the GPU they were shown on let go.
  pub fn hold(&mut self, scenes: impl IntoIterator<Item = RenderSceneId>) {
    let released: Instant = Instant::now();

    self.held.extend(scenes.into_iter().map(|scene| (scene, released)));
  }

  pub fn is_held(&self, scene: RenderSceneId) -> bool {
    self.held.contains_key(&scene)
  }

  /// Whether a held scene may be shown now, once it is whole or once it was held `WAIT_MOST`, letting it go then and
  /// logging how long it was held and, where it is not whole, what it still waits for.
  pub fn release(&mut self, scene: RenderSceneId, level: &LevelScene, textures: &TextureCache) -> bool {
    let Some(released) = self.held.get(&scene).copied() else {
      return true;
    };
    let waited: Duration = released.elapsed();
    let is_ready: bool = level.is_ready(textures);

    if !is_ready && waited < Self::WAIT_MOST {
      return false;
    }

    self.held.remove(&scene);

    match is_ready {
      true => log::info!(
        "Level shown again {} ms after the renderer restarted",
        waited.as_millis()
      ),
      false => log::warn!(
        "Level shown again {} ms after the renderer restarted, before it is whole: {}",
        waited.as_millis(),
        level.describe_pending(textures)
      ),
    }

    true
  }

  /// Lets go of the scenes the world no longer streams.
  pub fn retain(&mut self, live: &HashSet<RenderSceneId>) {
    self.held.retain(|scene, _| live.contains(scene));
  }
}
