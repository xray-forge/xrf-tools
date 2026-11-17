//! Which kind each surface of a scene's library objects is, told by its compiler shader and its texture, and a library
//! object parted by them into what the scene draws and what it shows on asking.

use xrf_editable_object::{EditableObjectFile, EditableSurface};
use xrf_rawdata::SurfaceDress;

use crate::scene::{SceneObjectParts, SceneSurfaceKind, SceneSurfaceKinds};
use crate::tests::fixtures::{COLLISION_COMPILER_SHADER, compiler_library, scene_parted_object};

/// The surface names each mesh of `part` holds faces of, and how many, mesh by mesh.
fn list_faces(part: &EditableObjectFile) -> Vec<(String, usize)> {
  part
    .meshes
    .iter()
    .flatten()
    .flat_map(|mesh| mesh.surface_faces.iter())
    .map(|surface_faces| (surface_faces.surface.clone(), surface_faces.faces.len()))
    .collect()
}

/// A surface of `compiler_shader` dressed in `texture`, the rest of it as a test object's.
fn surface(compiler_shader: &str, texture: &str) -> EditableSurface {
  EditableSurface {
    compiler_shader: String::from(compiler_shader),
    texture: String::from(texture),
    ..scene_parted_object(SurfaceDress::DEFAULT_SETTING)
      .surfaces
      .expect("the object has surfaces")[0]
      .clone()
  }
}

/// The test object whose second surface, `hull`, is of `compiler_shader` and dressed in `texture`.
fn parted(compiler_shader: &str, texture: &str) -> EditableObjectFile {
  let mut object: EditableObjectFile = scene_parted_object(compiler_shader);

  object.surfaces.as_mut().expect("the object has surfaces")[1].texture = String::from(texture);
  object
}

#[test]
fn a_surface_is_collision_by_its_compiler_shader_then_editor_only_by_its_texture() {
  let kinds: SceneSurfaceKinds = SceneSurfaceKinds::new(Some(compiler_library()));
  let kind = |compiler_shader: &str, texture: &str| kinds.get_kind(&surface(compiler_shader, texture));

  assert_eq!(
    kind(COLLISION_COMPILER_SHADER, "wood\\wood_planks"),
    SceneSurfaceKind::Collision
  );
  assert_eq!(
    kind("FLORA\\Flora_Collision", "wood\\wood_planks"),
    SceneSurfaceKind::Collision,
    "matched as the compiler matches a name"
  );
  assert_eq!(
    kind(SurfaceDress::DEFAULT_SETTING, "wood\\wood_planks"),
    SceneSurfaceKind::Drawn
  );
  assert_eq!(
    kind("def_shaders\\unknown", "wood\\wood_planks"),
    SceneSurfaceKind::Drawn,
    "a shader the library lacks is drawn"
  );
  assert_eq!(
    kind(SurfaceDress::DEFAULT_SETTING, "prop\\prop_fake_nosun"),
    SceneSurfaceKind::EditorOnly
  );
  assert_eq!(
    kind("def_shaders\\unknown", "Prop\\PROP_fake_occ"),
    SceneSurfaceKind::EditorOnly,
    "a marker matched as the game matches a name, whatever its shader"
  );
  assert_eq!(
    kind(COLLISION_COMPILER_SHADER, "prop\\prop_fake_kollision"),
    SceneSurfaceKind::Collision,
    "a collision hull's marker leaves it a collision surface"
  );
  assert_eq!(
    kind(SurfaceDress::DEFAULT_SETTING, "trees\\trees_kamish_sux_fake"),
    SceneSurfaceKind::Drawn,
    "only the prop markers, not every texture named fake"
  );
  assert_eq!(
    SceneSurfaceKinds::new(None).get_kind(&surface(COLLISION_COMPILER_SHADER, "wood\\wood_planks")),
    SceneSurfaceKind::Drawn,
    "with no library no surface is collision"
  );
}

#[test]
fn parts_an_object_into_the_kinds_asked_for_each_with_only_its_faces() {
  let kinds: SceneSurfaceKinds = SceneSurfaceKinds::new(Some(compiler_library()));
  let object: EditableObjectFile = scene_parted_object(COLLISION_COMPILER_SHADER);
  let drawn_surface: String = object.surfaces.as_ref().expect("the object has surfaces")[0]
    .name
    .clone();
  let mut parts: SceneObjectParts = kinds.split(object.clone(), &SceneSurfaceKind::ALL);
  let drawn: EditableObjectFile = parts
    .take(SceneSurfaceKind::Drawn)
    .expect("the drawn surface keeps a face");
  let collision: EditableObjectFile = parts.take(SceneSurfaceKind::Collision).expect("the hull keeps a face");

  assert_eq!(list_faces(&drawn), [(drawn_surface, 1)]);
  assert_eq!(list_faces(&collision), [(String::from("hull"), 1)]);
  assert_eq!(parts.take(SceneSurfaceKind::EditorOnly), None);
  assert_eq!(
    drawn.surfaces, object.surfaces,
    "each part keeps every surface, so it compiles as the object would"
  );
  assert_eq!(drawn.meshes.as_ref().map(|meshes| meshes[0].points.len()), Some(4));

  let mut asked: SceneObjectParts = kinds.split(object, &[SceneSurfaceKind::Collision]);

  assert_eq!(
    asked.take(SceneSurfaceKind::Drawn),
    None,
    "a kind not asked for is not parted"
  );
  assert!(asked.take(SceneSurfaceKind::Collision).is_some());
}

#[test]
fn keeps_whole_an_object_of_one_kind() {
  let kinds: SceneSurfaceKinds = SceneSurfaceKinds::new(Some(compiler_library()));
  let object: EditableObjectFile = scene_parted_object(SurfaceDress::DEFAULT_SETTING);

  assert_eq!(
    kinds.split(object.clone(), &SceneSurfaceKind::ALL),
    SceneObjectParts::whole(SceneSurfaceKind::Drawn, object)
  );

  let mut hull: EditableObjectFile = scene_parted_object(COLLISION_COMPILER_SHADER);

  for surface in hull.surfaces.iter_mut().flatten() {
    surface.compiler_shader = String::from(COLLISION_COMPILER_SHADER);
  }

  assert_eq!(
    kinds.split(hull.clone(), &SceneSurfaceKind::ALL),
    SceneObjectParts::whole(SceneSurfaceKind::Collision, hull)
  );

  let mut marked: EditableObjectFile = parted(SurfaceDress::DEFAULT_SETTING, "prop\\prop_fake_occ");

  for surface in marked.surfaces.iter_mut().flatten() {
    surface.texture = String::from("prop\\prop_fake_occ");
  }

  assert_eq!(
    kinds.split(marked.clone(), &SceneSurfaceKind::ALL),
    SceneObjectParts::whole(SceneSurfaceKind::EditorOnly, marked)
  );
}

#[test]
fn names_each_kinds_model_after_its_reference_and_reads_it_back() {
  for kind in SceneSurfaceKind::ALL {
    let name: String = kind.format_model_name("trees\\trees_pine_01");

    assert_eq!(SceneSurfaceKind::of_model(&name), kind);
    assert_eq!(kind.get_reference(&name), "trees\\trees_pine_01");
  }

  assert_eq!(
    SceneSurfaceKind::Drawn.format_model_name("trees\\trees_pine_01"),
    "trees\\trees_pine_01"
  );
}
