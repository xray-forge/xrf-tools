//! What `ObjectVerifier` counts as a difference, each reference here compiled from the object with one thing changed.

use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableBoneMass, EditableObjectFile};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_ogf::{OgfFile, OgfVertex, OgfVertexLinks};

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::decompile::tests::fixtures::{UP, assemble, chain, corner, object, surface, with_links};
use crate::object::verify::object_verification::{ObjectGeometryCensus, ObjectVerification};
use crate::object::verify::object_verifier::ObjectVerifier;
use crate::object::verify::ogf_child_geometry::{OgfChildGeometry, PositionKey};

/// A quad on a chain of three bones, one corner blended between the first two, with LODs and motion references.
fn quad() -> XrfResult<EditableObjectFile> {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
  let mut quad: EditableObjectFile = object(
    assemble(&[surface(vec![
      [corner(a, UP), corner(b, UP), corner(c, UP)],
      [
        corner(a, UP),
        corner(c, UP),
        with_links(corner(d, UP), &[(0, 0.5), (1, 0.5)]),
      ],
    ])])?,
    chain(),
  );

  quad.lods = Some(String::from("dynamics\\crate_lod"));
  quad.motion_references = Some(vec![String::from("crate_motions")]);

  Ok(quad)
}

/// `object` compiled, as a reference read from disk would be.
fn compiled(object: &EditableObjectFile) -> XrfResult<OgfFile> {
  let source: ObjectCompileSource = ObjectCompileSource {
    source_file: String::from("props\\crate.object"),
    convertor: String::from("SDK"),
    built_at: 1_000,
  };

  OgfFile::read_from_bytes::<XRayByteOrder>(ObjectCompiler::compile(object, &source)?)
}

/// How the quad verifies against a reference compiled from it after `change`.
fn verify_against(change: impl FnOnce(&mut EditableObjectFile)) -> XrfResult<ObjectVerification> {
  let mut changed: EditableObjectFile = quad()?;

  change(&mut changed);

  ObjectVerifier::verify(&quad()?, &compiled(&changed)?)
}

#[test]
fn finds_nothing_against_its_own_export() -> XrfResult {
  let verification: ObjectVerification = verify_against(|_| {})?;

  assert!(verification.is_equivalent(), "{verification:?}");
  assert_eq!(
    verification.census,
    ObjectGeometryCensus {
      children: 1,
      reference_children: 1,
      vertices: 6,
      vertices_matched: 6,
      vertices_normal_only: 0,
      faces: 2,
      reference_faces: 2,
      faces_matched: 2,
      reference_flat_faces: 0,
    }
  );

  Ok(())
}

#[test]
fn finds_a_renamed_bone_and_changed_physics() -> XrfResult {
  let verification: ObjectVerification = verify_against(|object| {
    let bones = object.bones.as_mut().expect("the quad has bones");

    bones[2].definition.name = String::from("tip");
    bones[1].mass = Some(EditableBoneMass {
      mass: 5.0,
      center: Vector3d::ZERO,
    });
  })?;

  assert_eq!(verification.skeleton.len(), 2, "{:?}", verification.skeleton);
  assert!(
    verification.skeleton[0].starts_with("bone 2: "),
    "{:?}",
    verification.skeleton
  );
  assert!(verification.skeleton[1].contains("mass"), "{:?}", verification.skeleton);
  assert!(verification.model.is_empty() && verification.motions.is_empty() && verification.geometry.is_empty());

  Ok(())
}

#[test]
fn finds_what_the_model_is_to_the_engine() -> XrfResult {
  let verification: ObjectVerification = verify_against(|object| {
    object.class_script = Some(String::from("[collide]\nignore_static"));
    object.lods = Some(String::from("dynamics\\crate_lod_2"));
  })?;

  assert_eq!(verification.model.len(), 2, "{:?}", verification.model);
  assert!(verification.model[0].starts_with("user data: "));
  assert!(verification.model[1].starts_with("LODs: "));

  Ok(())
}

#[test]
fn finds_a_child_drawn_with_another_texture() -> XrfResult {
  let verification: ObjectVerification = verify_against(|object| {
    object.surfaces.as_mut().expect("the quad has surfaces")[0].texture = String::from("wood\\wood_plank7");
  })?;

  assert_eq!(
    verification.geometry,
    [
      "child wood\\wood_plank6 / models\\model has no reference child",
      "the reference's child wood\\wood_plank7 / models\\model is not compiled",
    ]
  );
  assert_eq!(verification.census.faces_matched, 0);

  Ok(())
}

#[test]
fn finds_a_moved_corner_in_its_vertices_faces_and_bounds() -> XrfResult {
  let verification: ObjectVerification = verify_against(|object| {
    let mesh = &mut object.meshes.as_mut().expect("the quad has a mesh")[0];
    let far: usize = mesh
      .points
      .iter()
      .position(|point| *point == Vector3d::new(1.0, 0.0, 1.0))
      .expect("the quad has a far corner");

    mesh.points[far] = Vector3d::new(1.0, 0.0, 2.0);
    mesh.bounds.max = Vector3d::new(1.0, 0.0, 2.0);
  })?;

  let geometry: &[String] = &verification.geometry;

  assert_eq!(geometry.len(), 3, "{geometry:?}");
  assert!(geometry[0].starts_with("bounds "), "{geometry:?}");
  assert!(
    geometry[1]
      .ends_with("2 of 6 vertices have no reference vertex like them, 0 of them differing in their normal alone"),
    "{geometry:?}"
  );
  assert!(
    geometry[2].ends_with("2 reference faces not drawn, 2 drawn the reference does not"),
    "{geometry:?}"
  );

  Ok(())
}

#[test]
fn leaves_out_a_face_naming_a_vertex_the_child_lacks() {
  let vertex = |x: f32| OgfVertex {
    position: Vector3d::new(x, 0.0, 0.0),
    normal: Vector3d::new(0.0, 1.0, 0.0),
    tangent: Vector3d::ZERO,
    binormal: Vector3d::ZERO,
    texture_u: 0.0,
    texture_v: 0.0,
    links: OgfVertexLinks::default(),
  };
  let vertices: [OgfVertex; 3] = [vertex(0.0), vertex(1.0), vertex(2.0)];
  let child: OgfChildGeometry = OgfChildGeometry {
    texture: "wood",
    shader: "models\\model",
    vertices: &vertices,
    faces: vec![[2, 0, 1], [0, 1, 3]],
  };
  let key = |x: f32| OgfChildGeometry::position_key(&Vector3d::new(x, 0.0, 0.0));
  let mut expected: [PositionKey; 3] = [key(0.0), key(1.0), key(2.0)];

  expected.sort_unstable();

  assert_eq!(child.triangles(), [expected]);
}
