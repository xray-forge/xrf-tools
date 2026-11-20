use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_math::Sphere;
use xrf_vfs::XraySearch;

use crate::scene::{
  SceneDrawPart, SceneHelpersPart, SceneObjectId, SceneSpawnModel, SceneSpawnModels, SceneSpawnModelsPart,
  SceneSpawnVisualsPart, SceneToolKind, SelectedScene,
};

/// Where a scene object stands for a camera to frame it, answered from the scene session's parts.
pub(crate) struct SceneObjectSphere;

impl SceneObjectSphere {
  /// The sphere about `object` as drawn, in renderer space: a scene object's placed box; a spawn model's declared box,
  /// placed; or a helped object's helper extent. None for an object not drawn, such as a sector or a group.
  ///
  /// A spawn model is read through the scene's kept visuals, which it shares with the renderer either way; one that
  /// does not read is located by its helper.
  ///
  /// # Errors
  ///
  /// Returns an error when the scene's objects roots or game data cannot be searched.
  pub(crate) fn locate(
    current: &SessionSnapshot<SelectedScene>,
    assets: &AssetMountState,
    object: &SceneObjectId,
  ) -> TauriResult<Option<Sphere>> {
    if object.tool == SceneToolKind::SceneObject {
      return Ok(SceneDrawPart::get(current, &assets.search(&current.objects_roots)?).find_sphere(object.index));
    }

    let game: XraySearch = assets.search(&current.game.roots)?;

    if object.tool == SceneToolKind::Spawn
      && let Some(sphere) = Self::locate_model(current, &game, object.index)
    {
      return Ok(Some(sphere));
    }

    Ok(SceneHelpersPart::get(current, &game).find_sphere(object))
  }

  /// The sphere about the placed model of the spawn point at `index`; none for one not drawn as a model, or whose
  /// model does not read.
  fn locate_model(current: &SessionSnapshot<SelectedScene>, game: &XraySearch, index: u32) -> Option<Sphere> {
    let models: Arc<SceneSpawnModels> = SceneSpawnModelsPart::get(current, game);
    let model: &SceneSpawnModel = models.find(index)?;

    SceneSpawnVisualsPart::with_reader(current, game, |reader| reader.get(&model.visual))
      .ok()
      .map(|visual| model.place_bounds(&visual.package.description.declared_bounds))
  }
}
