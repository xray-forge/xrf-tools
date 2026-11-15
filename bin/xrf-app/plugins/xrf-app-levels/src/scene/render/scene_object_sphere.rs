use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_math::Sphere;

use crate::scene::{SceneDrawPart, SceneHelpersPart, SceneObjectId, SceneToolKind, SelectedScene};

/// Where a scene object stands for a camera to frame it, answered from the parts the scene's session keeps.
pub(crate) struct SceneObjectSphere;

impl SceneObjectSphere {
  /// The sphere about `object` as drawn, in renderer space: a scene object's placed box, read through `assets` the
  /// first time the scene's drawing is asked for, or the extent of a helped object's helper; none for an object that
  /// draws nothing, a sector or a group, or that is not drawn.
  ///
  /// # Errors
  ///
  /// Returns an error when the scene's objects roots cannot be searched for a scene object.
  pub(crate) fn locate(
    current: &SessionSnapshot<SelectedScene>,
    assets: &AssetMountState,
    object: &SceneObjectId,
  ) -> TauriResult<Option<Sphere>> {
    Ok(match object.tool {
      SceneToolKind::SceneObject => {
        SceneDrawPart::get(current, &assets.search(&current.objects_roots)?).find_sphere(object.index)
      }
      _ => SceneHelpersPart::get(current).find_sphere(object),
    })
  }
}
