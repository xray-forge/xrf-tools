use std::collections::HashMap;
use std::sync::Arc;

use xrf_math::Sphere;

use crate::scene::SceneDrawDescription;

/// A scene's drawn objects as its session keeps them: what the page is told of them, and where each stands for a
/// camera to frame it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneDraw {
  pub description: Arc<SceneDrawDescription>,
  /// The sphere about each drawn object's placed box, in renderer space, by the object's index among the scene's
  /// objects; none for an object whose library object has no mesh.
  pub spheres: HashMap<u32, Sphere>,
}

impl SceneDraw {
  /// The sphere about the scene object at `index`, in renderer space; none for one not drawn.
  pub(crate) fn find_sphere(&self, index: u32) -> Option<Sphere> {
    self.spheres.get(&index).copied()
  }
}
