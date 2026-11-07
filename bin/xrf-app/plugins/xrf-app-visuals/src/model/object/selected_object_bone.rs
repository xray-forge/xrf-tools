use serde::Serialize;
use xrf_editable_object::EditableBone;

/// One bone of an object as it was authored: its physics as the Actor Editor sets them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedObjectBone {
  pub name: String,
  /// The bone it hangs from, empty for the root.
  pub parent: String,
  pub game_material: String,
  /// The shape it collides as: `None`, `Box`, `Sphere` or `Cylinder`.
  pub shape: String,
  /// How it joins its parent: `Rigid`, `Cloth`, `Joint`, `Wheel`, `None` or `Slider`.
  pub joint: String,
  /// Its mass in kilograms, or none where the object stores none.
  pub mass: Option<f32>,
}

impl SelectedObjectBone {
  pub fn of(bone: &EditableBone) -> Self {
    Self {
      name: bone.definition.name.clone(),
      parent: bone.definition.parent.clone(),
      game_material: bone.game_material.clone(),
      shape: format!("{:?}", bone.shape.kind),
      joint: format!("{:?}", bone.joint.kind),
      mass: bone.mass.as_ref().map(|mass| mass.mass),
    }
  }
}
