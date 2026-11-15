use glam::{Mat4, Vec3};
use xrf_editable_level::EditableObjectTransform;
use xrf_extension::XrayExtension;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_visual::VisualBounds;

use crate::scene::{SceneObjectId, SceneRenderSpace};

/// One spawn point the scene draws as its entity's model: the visual its packet names, found in the scene's game data,
/// and where the spawn point stands.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneSpawnModel {
  pub id: SceneObjectId,
  /// The visual its entity's packet names, as the game data's meshes hold it.
  pub visual: String,
  pub transform: EditableObjectTransform,
}

impl SceneSpawnModel {
  /// The name the renderer knows the model of `visual` by: its file's, so no library object's reference spells it.
  pub(crate) fn format_model_name(visual: &str) -> String {
    format!("{visual}.{}", XrayExtension::Ogf.as_str())
  }

  /// The name the renderer knows its model by.
  pub(crate) fn get_model_name(&self) -> String {
    Self::format_model_name(&self.visual)
  }

  /// Where it stands as the renderer places its model, sixteen floats column by column: the spawn point's transform in
  /// renderer space, as a scene object's is.
  pub(crate) fn get_render_transform(&self) -> [f32; 16] {
    SceneRenderSpace::place(&self.transform)
  }

  /// The sphere about the box `bounds` reach, a model's own in renderer space, once placed where it stands.
  pub(crate) fn place_bounds(&self, bounds: &VisualBounds) -> Sphere {
    let placement: Mat4 = Mat4::from_cols_array(&self.get_render_transform());
    let Aabb { min, max } = bounds.bounding_box;
    let place = |corner: usize| -> Vector3d {
      let placed: Vec3 = placement.transform_point3(Vec3::new(
        if corner & 1 == 0 { min.x } else { max.x },
        if corner & 2 == 0 { min.y } else { max.y },
        if corner & 4 == 0 { min.z } else { max.z },
      ));

      Vector3d::new(placed.x, placed.y, placed.z)
    };
    let mut placed: Aabb = Aabb::of_point(&place(0));

    for corner in 1..8 {
      placed.include_point(&place(corner));
    }

    placed.get_bounding_sphere()
  }
}
