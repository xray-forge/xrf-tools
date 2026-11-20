// The compile itself is `xrf-rawdata`'s to pin, the packing `xrf-visual`'s.

use xrf_editable_object::EditableObjectFile;
use xrf_math::{Aabb, Vector3d};
use xrf_ogf::OgfModelType;
use xrf_rawdata::ObjectCompileOptions;
use xrf_visual::{VisualBounds, VisualSubmeshContent};

use crate::ObjectVisual;
use crate::tests::fixtures::{compile_source, object_of, quad, surface};

fn compile(object: &EditableObjectFile) -> ObjectVisual {
  ObjectVisual::compile(object, &compile_source(), &ObjectCompileOptions::default()).expect("the object compiles")
}

#[test]
fn packs_a_static_objects_parts_in_renderer_space_with_a_submesh_a_surface() {
  let object: EditableObjectFile = object_of(&[
    surface("models\\model", "wood\\planks", &quad(2.0, 3.0)),
    surface("models\\window", "glass\\pane", &quad(1.0, 1.0)),
  ]);
  let visual: ObjectVisual = compile(&object);
  let bounds: &VisualBounds = &visual.package.description.declared_bounds;

  assert_eq!(
    visual
      .package
      .description
      .submeshes
      .iter()
      .map(|submesh| (submesh.shader_name.as_deref(), submesh.texture_name.as_deref()))
      .collect::<Vec<_>>(),
    [
      (Some("models\\model"), Some("wood\\planks")),
      (Some("models\\window"), Some("glass\\pane")),
    ]
  );
  assert!(
    visual
      .package
      .description
      .submeshes
      .iter()
      .all(|submesh| matches!(submesh.content, VisualSubmeshContent::Packed { .. })),
    "every submesh draws: {:?}",
    visual
      .package
      .description
      .submeshes
      .iter()
      .map(|submesh| match &submesh.content {
        VisualSubmeshContent::Packed { .. } => String::from("packed"),
        VisualSubmeshContent::Skipped { reason, .. } => reason.clone(),
      })
      .collect::<Vec<_>>()
  );
  // The extent the compiled file declares, in renderer space, whose `z` runs the other way.
  assert_eq!(
    bounds.bounding_box,
    Aabb {
      min: Vector3d::new(0.0, 0.0, -3.0),
      max: Vector3d::new(2.0, 0.0, 0.0),
    }
  );
  assert_eq!(
    visual.package.description.buffer_length as usize,
    visual.package.buffer.len()
  );
  assert_eq!(
    visual.package.description.model_type,
    OgfModelType::Hierarchy.to_raw(),
    "described as its file declares it, a part a surface"
  );
  assert!(visual.motions.is_none(), "a static object has no bind pose");
}

#[test]
fn refuses_an_object_holding_no_mesh() {
  let mut object: EditableObjectFile = object_of(&[surface("models\\model", "wood\\planks", &quad(1.0, 1.0))]);

  object.meshes = Some(Vec::new());

  assert!(ObjectVisual::compile(&object, &compile_source(), &ObjectCompileOptions::default()).is_err());
}
