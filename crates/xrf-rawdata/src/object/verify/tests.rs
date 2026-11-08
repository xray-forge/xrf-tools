//! What `ObjectVerifier` counts as a difference, each reference here compiled from the object with one thing changed.

use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableBone, EditableBoneMass, EditableObjectFile};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_ogf::{OgfFile, OgfVertex, OgfVertexLinks};

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::decompile::tests::fixtures::{UP, assemble, chain, corner, object, surface, with_links};
use crate::object::verify::material_vertex::MaterialVertex;
use crate::object::verify::object_verification::{ObjectGeometryCensus, ObjectVerification};
use crate::object::verify::object_verifier::ObjectVerifier;
use crate::object::verify::ogf_material_geometry::{OgfMaterialComparison, OgfMaterialGeometry};

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

  OgfFile::read_from_bytes::<XRayByteOrder>(ObjectCompiler::compile(object, &source)?.bytes)
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
fn takes_a_stored_nan_for_itself() -> XrfResult {
  // Vanilla bones store a NaN friction, which a round trip keeps and plain equality never finds equal.
  let mut object: EditableObjectFile = quad()?;

  object.bones.as_mut().expect("the quad has bones")[0].friction = Some(f32::NAN);

  let verification: ObjectVerification = ObjectVerifier::verify(&object, &compiled(&object)?)?;

  assert!(verification.skeleton.is_empty(), "{:?}", verification.skeleton);

  Ok(())
}

#[test]
fn reads_user_data_up_to_its_terminators() -> XrfResult {
  // Exporters end the user data with a terminator or not; the engine reads the same ini text either way.
  let mut object: EditableObjectFile = quad()?;

  object.class_script = Some(String::from("[collide]\nignore_static"));

  let mut exported: EditableObjectFile = object.clone();

  exported.class_script = Some(String::from("[collide]\nignore_static\0"));

  let verification: ObjectVerification = ObjectVerifier::verify(&object, &compiled(&exported)?)?;

  assert!(verification.model.is_empty(), "{:?}", verification.model);

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
      "wood\\wood_plank6 / models\\model is not drawn by the reference",
      "the reference's wood\\wood_plank7 / models\\model is not compiled",
    ]
  );
  assert_eq!(verification.census.faces_matched, 0);

  Ok(())
}

#[test]
fn takes_a_surface_split_by_breakable_part_for_the_one_child_it_was() -> XrfResult {
  // One triangle on the root and one on the lid: with the lid breakable the export draws them as two children, which
  // an older exporter or another tool kept as one; the engine draws either alike.
  let triangles = |offset: f32| {
    let at = |x: f32, z: f32| [x + offset, 0.0, z];

    [
      corner(at(0.0, 0.0), UP),
      corner(at(1.0, 0.0), UP),
      corner(at(1.0, 1.0), UP),
    ]
  };
  let [first, second] = [triangles(0.0), triangles(2.0).map(|it| with_links(it, &[(1, 1.0)]))];
  let mesh = || assemble(&[surface(vec![first, second])]);
  let mut bones: Vec<EditableBone> = chain();

  bones[1].joint_flags = Some(1);

  let verification: ObjectVerification =
    ObjectVerifier::verify(&object(mesh()?, bones), &compiled(&object(mesh()?, chain()))?)?;

  assert!(verification.geometry.is_empty(), "{:?}", verification.geometry);
  assert_eq!(
    (
      verification.census.children,
      verification.census.reference_children,
      verification.census.faces_matched
    ),
    (2, 1, 2)
  );

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

/// A vertex at `x` along the X axis, facing up, at the origin of its texture, following no bone.
fn vertex(x: f32) -> OgfVertex {
  OgfVertex {
    position: Vector3d::new(x, 0.0, 0.0),
    normal: Vector3d::new(0.0, 1.0, 0.0),
    tangent: Vector3d::ZERO,
    binormal: Vector3d::ZERO,
    texture_u: 0.0,
    texture_v: 0.0,
    links: OgfVertexLinks::default(),
  }
}

#[test]
fn compares_no_face_that_draws_nothing_on_either_side() {
  // The fourth vertex stands where the first does, as `VPack` leaves a corner it moved onto another: a face on both
  // draws nothing, so one kept by either side alone is no difference.
  let vertices: [OgfVertex; 4] = [vertex(0.0), vertex(1.0), vertex(2.0), vertex(0.0)];
  let material = |faces: Vec<[u32; 3]>| OgfMaterialGeometry {
    texture: "wood",
    shader: "models\\model",
    children: 1,
    vertices: vertices.iter().map(MaterialVertex::from).collect(),
    faces,
  };
  let with_flat: OgfMaterialGeometry<'_> = material(vec![[0, 1, 2], [0, 3, 1]]);
  let without: OgfMaterialGeometry<'_> = material(vec![[0, 1, 2]]);

  for (compiled, reference) in [(&with_flat, &without), (&without, &with_flat)] {
    let comparison: OgfMaterialComparison = compiled.compare(reference);

    assert_eq!(
      (
        comparison.faces_matched,
        comparison.faces_missing,
        comparison.faces_extra
      ),
      (1, 0, 0)
    );
  }

  assert_eq!(without.compare(&with_flat).reference_flat_faces, 1);
}

#[test]
fn leaves_out_a_face_naming_a_vertex_it_lacks() {
  let vertices: [OgfVertex; 3] = [vertex(0.0), vertex(1.0), vertex(2.0)];
  let material: OgfMaterialGeometry<'_> = OgfMaterialGeometry {
    texture: "wood",
    shader: "models\\model",
    children: 1,
    vertices: vertices.iter().map(MaterialVertex::from).collect(),
    faces: vec![[2, 0, 1], [0, 1, 3]],
  };
  let key = |x: f32| OgfMaterialGeometry::position_key(&Vector3d::new(x, 0.0, 0.0));

  // Turned to start at its lowest corner, its winding kept.
  assert_eq!(material.triangles(), [[key(0.0), key(1.0), key(2.0)]]);
}

#[test]
fn compares_each_face_once_and_its_mirror_apart() {
  let vertices: [OgfVertex; 3] = [vertex(0.0), vertex(1.0), vertex(2.0)];
  let material = |faces: Vec<[u32; 3]>| OgfMaterialGeometry {
    texture: "wood",
    shader: "models\\model",
    children: 1,
    vertices: vertices.iter().map(MaterialVertex::from).collect(),
    faces,
  };
  // The static export's second walk of a two-sided surface: each face and its mirror written again.
  let repeated: OgfMaterialGeometry<'_> = material(vec![[0, 1, 2], [2, 1, 0], [1, 2, 0], [0, 2, 1]]);
  let once: OgfMaterialGeometry<'_> = material(vec![[0, 1, 2], [2, 1, 0]]);
  let one_side: OgfMaterialGeometry<'_> = material(vec![[0, 1, 2]]);
  let counts = |comparison: OgfMaterialComparison| {
    (
      comparison.faces_matched,
      comparison.faces_missing,
      comparison.faces_extra,
    )
  };

  assert_eq!(repeated.triangles().len(), 2);
  assert_eq!(counts(once.compare(&repeated)), (2, 0, 0));
  assert_eq!(counts(one_side.compare(&repeated)), (1, 1, 0));
}
