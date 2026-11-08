use xrf_editable_object::{EditableDescription, EditableObjectFile, EditableSurface, EditableTimeWidth};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector3d};
use xrf_ogf::{OgfFile, OgfGeometry, OgfModelType};

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::compile::tests::ogf::{authored, compile};
use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::object_inference::ObjectInference;
use crate::object::decompile::tests::fixtures::{UP, assemble_static, corner, static_object, surface, unweighted};
use crate::object::verify::object_verifier::ObjectVerifier;

/// A flat quad of side one at height `y`, its two faces sharing the edge `a c`, every corner following no bone.
fn quad(y: f32) -> Vec<[OgfCorner; 3]> {
  let [a, b, c, d] = [[0.0, y, 0.0], [1.0, y, 0.0], [1.0, y, 1.0], [0.0, y, 1.0]];
  let at = |position: [f32; 3]| unweighted(corner(position, UP));

  vec![[at(a), at(b), at(c)], [at(a), at(c), at(d)]]
}

/// A surface of `triangles` drawn with `texture`.
fn textured(texture: &str, triangles: Vec<[OgfCorner; 3]>) -> OgfSurface {
  OgfSurface {
    texture: String::from(texture),
    ..surface(triangles)
  }
}

/// The geometry a static visual holds.
fn geometry(file: &OgfFile) -> &OgfGeometry {
  file.geometry.as_ref().expect("Expect the visual's geometry")
}

#[test]
fn writes_one_surface_as_a_normal_visual_with_its_collision_mesh() -> XrfResult {
  let file: OgfFile = compile(&static_object(assemble_static(&[surface(quad(0.0))])?))?;

  assert!(file.residue.is_none(), "{:?}", file.residue);
  assert_eq!(file.header.model_type, OgfModelType::Normal.to_raw());
  assert_eq!(
    file.header.bounding_box,
    Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::new(1.0, 0.0, 1.0)
    }
  );
  // A single part carries neither children nor the object's description (`ExportObjectOGF.cpp`, `SavePart`).
  assert!(file.children.is_none());
  assert!(file.get_description().is_none());
  assert_eq!(
    file
      .texture
      .as_ref()
      .map(|it| (it.texture_name.as_str(), it.shader_name.as_str())),
    Some(("wood\\wood_plank6", "models\\model"))
  );
  assert_eq!(
    geometry(&file).vertex_format,
    Some(EditableSurface::POSITION_NORMAL_UV_FVF)
  );
  assert_eq!(geometry(&file).get_static_vertices().map(<[_]>::len), Some(4));
  assert_eq!(geometry(&file).indices.as_ref().map(Vec::len), Some(6));
  assert_eq!(file.collision_vertices.as_ref().map(|it| it.vertices.len()), Some(4));
  assert_eq!(file.collision_indices.as_ref().map(|it| it.faces.len()), Some(2));

  Ok(())
}

#[test]
fn writes_a_surface_each_as_the_parts_of_a_hierarchy() -> XrfResult {
  let mut object: EditableObjectFile = static_object(assemble_static(&[
    textured("wood\\wood_plank6", quad(0.0)),
    textured("wood\\wood_plank7", quad(1.0)),
  ])?);

  object.description = Some(EditableDescription {
    creator: String::from("maker"),
    created_at: 1_000,
    modifier: String::from("editor"),
    modified_at: 2_000,
    time_width: EditableTimeWidth::Seconds32,
  });

  let file: OgfFile = compile(&object)?;
  let children: &[OgfFile] = &file.children.as_ref().expect("Expect children").nested;

  assert_eq!(file.header.model_type, OgfModelType::Hierarchy.to_raw());
  assert_eq!(file.header.bounding_box.max, Vector3d::new(1.0, 1.0, 1.0));
  assert_eq!(file.get_description().map(|it| it.created_at), Some(1_000));
  assert_eq!(
    children
      .iter()
      .map(|child| (
        child.header.model_type,
        child.texture.as_ref().map(|it| it.texture_name.as_str())
      ))
      .collect::<Vec<_>>(),
    [
      (OgfModelType::Normal.to_raw(), Some("wood\\wood_plank6")),
      (OgfModelType::Normal.to_raw(), Some("wood\\wood_plank7")),
    ]
  );

  // The collision mesh numbers the second part's vertices past the first's.
  let faces = &file.collision_indices.as_ref().expect("Expect collision faces").faces;

  assert_eq!(faces.len(), 4);
  assert!(
    faces[2..]
      .iter()
      .all(|face| face.vertices.iter().all(|vertex| *vertex >= 4))
  );

  Ok(())
}

#[test]
fn writes_a_two_sided_face_once_with_its_mirror() -> XrfResult {
  let mut object: EditableObjectFile = static_object(assemble_static(&[surface(quad(0.0))])?);

  object.surfaces.as_mut().expect("Expect surfaces")[0].flags = EditableSurface::TWO_SIDED_FLAG;

  let file: OgfFile = compile(&object)?;
  let vertices = geometry(&file).get_static_vertices().expect("Expect static vertices");

  // Each face and its mirror once, where the export's second walk of the surface writes both again; the mirror's
  // corners face down.
  assert_eq!(geometry(&file).indices.as_ref().map(Vec::len), Some(12));
  assert_eq!(vertices.len(), 8);
  assert_eq!(
    vertices
      .iter()
      .filter(|vertex| vertex.normal == Some(-&Vector3d::Y))
      .count(),
    4
  );

  let decompiled: EditableObjectFile = ObjectDecompiler::decompile(&file)?.object;
  let surfaces: &[EditableSurface] = decompiled.surfaces.as_deref().expect("Expect surfaces");

  assert_eq!(decompiled.meshes.as_ref().map(|it| it[0].faces.len()), Some(2));
  assert!(surfaces.len() == 1 && surfaces[0].is_two_sided());

  Ok(())
}

#[test]
fn compiles_a_dynamic_object_without_bones_as_a_static_visual() -> XrfResult {
  // The export chooses by bones alone (`CEditableObject::IsSkeleton`), as for the SDK's bone-less `dm_lightning`.
  let object: EditableObjectFile = EditableObjectFile {
    flags: EditableObjectFile::DYNAMIC_FLAG,
    ..static_object(assemble_static(&[surface(quad(0.0))])?)
  };

  assert_eq!(compile(&object)?.header.model_type, OgfModelType::Normal.to_raw());

  Ok(())
}

#[test]
fn refuses_a_face_naming_a_point_its_mesh_lacks_in_either_export() -> XrfResult {
  let broken = |mut object: EditableObjectFile| -> String {
    object.meshes.as_mut().expect("Expect a mesh")[0].faces[0].corners[1].point = 999;

    ObjectCompiler::compile(
      &object,
      &ObjectCompileSource {
        source_file: String::new(),
        convertor: String::new(),
        built_at: 0,
      },
    )
    .expect_err("Expect the broken object refused")
    .to_string()
  };

  for error in [
    broken(static_object(assemble_static(&[surface(quad(0.0))])?)),
    broken(authored()?),
  ] {
    assert!(error.contains("names point 999"), "{error}");
  }

  Ok(())
}

#[test]
fn decompiles_a_static_visual_into_a_static_object_its_compile_stands_in_for() -> XrfResult {
  for surfaces in [
    vec![surface(quad(0.0))],
    vec![
      textured("wood\\wood_plank6", quad(0.0)),
      textured("wood\\wood_plank7", quad(1.0)),
    ],
  ] {
    let file: OgfFile = compile(&static_object(assemble_static(&surfaces)?))?;
    let decompile = ObjectDecompiler::decompile(&file)?;
    let object: &EditableObjectFile = &decompile.object;
    let mesh = &object.meshes.as_ref().expect("Expect a mesh")[0];

    assert_eq!(object.flags, 0);
    assert!(object.bones.is_none());
    assert_eq!(mesh.name, "mesh");
    assert!(
      mesh
        .vmaps
        .iter()
        .flatten()
        .all(|vmap| vmap.name == EditableSurface::UV_MAP_NAME)
    );
    assert_eq!(object.surfaces.as_ref().map(Vec::len), Some(surfaces.len()));
    assert!(!decompile.inferred.contains(&ObjectInference::BoneLengths));

    let verification = ObjectVerifier::verify(object, &file)?;

    assert!(verification.is_equivalent(), "{verification:?}");
  }

  Ok(())
}
