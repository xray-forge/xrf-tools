use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface};
use xrf_error::XrfResult;
use xrf_level::{DetailModel, DetailVertex};
use xrf_math::Vector3d;

use crate::object::compile::object_compile::ObjectCompile;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::tests::fixtures::{
  UP, assemble_static, corner, static_object, surface, unweighted, with_uv,
};
use crate::object::detail_parameters::DetailParameters;

fn vertex(position: [f32; 3], u: f32, v: f32) -> DetailVertex {
  DetailVertex {
    position: Vector3d::from(position),
    u,
    v,
  }
}

/// A leaf of two triangles meeting along `a c`, its corner `c` mapped differently on either side, as a seam stores one
/// point twice; the vertices in the order the triangles first reach them, as the export leaves them.
fn leaf(parameters: &DetailParameters) -> DetailModel {
  DetailModel {
    shader: String::from("details\\blend"),
    texture: String::from("det\\det_listya"),
    flags: parameters.flags,
    min_scale: parameters.min_scale,
    max_scale: parameters.max_scale,
    vertices: vec![
      vertex([0.0, 0.0, 0.0], 0.0, 1.0),
      vertex([1.0, 0.0, 0.0], 1.0, 1.0),
      vertex([1.0, 1.0, 0.0], 1.0, 0.0),
      vertex([1.0, 1.0, 0.0], 0.5, 0.0),
      vertex([0.0, 1.0, 0.0], 0.0, 0.0),
    ],
    indices: vec![0, 1, 2, 0, 3, 4],
  }
}

fn to_bytes(model: &DetailModel) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  model.write::<XRayByteOrder>(&mut writer)?;
  writer.flush_raw_into_buffer()
}

fn compile(object: &EditableObjectFile) -> XrfResult<ObjectCompile> {
  ObjectCompiler::compile_detail(object, &DetailParameters::default())
}

fn read(compiled: &ObjectCompile) -> XrfResult<DetailModel> {
  DetailModel::read_from_bytes::<XRayByteOrder>(compiled.bytes.clone())
}

#[test]
fn decompiles_a_detail_model_and_compiles_it_back_byte_for_byte() -> XrfResult {
  let parameters: DetailParameters = DetailParameters {
    flags: 1,
    min_scale: 0.8,
    max_scale: 1.3,
  };
  let model: DetailModel = leaf(&parameters);
  let decompile: ObjectDecompile = ObjectDecompiler::decompile_detail(&model)?;
  let surfaces: &[EditableSurface] = decompile.object.surfaces.as_deref().expect("Expect surfaces");
  let mesh: &EditableMesh = &decompile.object.meshes.as_ref().expect("Expect a mesh")[0];

  assert_eq!(decompile.detail, Some(parameters));
  assert_eq!(
    (
      surfaces.len(),
      surfaces[0].shader.as_str(),
      surfaces[0].texture.as_str()
    ),
    (1, "details\\blend", "det\\det_listya")
  );
  // The seam's point is one, mapped per face; the model stores no normal, so the mesh keeps none for its corners.
  assert_eq!(mesh.points.len(), 4);
  assert!(mesh.corner_normals.is_none());

  // Written and read back as the SDK would find it, the object compiles to the model it came from.
  let object: EditableObjectFile =
    EditableObjectFile::read_from_bytes::<XRayByteOrder>(decompile.object.write_to_bytes::<XRayByteOrder>()?)?;
  let compiled: ObjectCompile = ObjectCompiler::compile_detail(&object, &parameters)?;

  assert_eq!(compiled.bytes, to_bytes(&model)?);
  assert_eq!((compiled.vertices, compiled.faces, compiled.dropped_faces), (5, 2, 0));

  Ok(())
}

#[test]
fn drops_degenerate_and_repeated_faces_a_mirror_among_them_and_merges_near_corners() -> XrfResult {
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  // Half a millimetre from `a`, mapped alike: within `EPS_L` of it, so the same vertex.
  let near_a: [f32; 3] = [0.000_5, 0.0, 0.0];
  let at = |position: [f32; 3], normal: [f32; 3]| unweighted(corner(position, normal));
  let triangles: Vec<[OgfCorner; 3]> = vec![
    [at(a, UP), at(b, UP), at(c, UP)],
    // Its mirror, the same three vertices in another order; facing up still, so the fixture's own assembly keeps it a
    // face of its own rather than folding it two-sided.
    [at(c, UP), at(b, UP), at(a, UP)],
    // The first again, starting at another corner, through the near copy of `a`.
    [at(b, UP), at(c, UP), at(near_a, UP)],
    // A face whose corners `a` and its near copy are one vertex.
    [at(a, UP), at(near_a, UP), at(b, UP)],
  ];
  let compiled: ObjectCompile = compile(&static_object(assemble_static(&[surface(triangles)])?))?;
  let model: DetailModel = read(&compiled)?;

  assert_eq!(model.vertices.len(), 3);
  assert_eq!(model.indices, [0, 1, 2]);
  assert_eq!(compiled.dropped_faces, 3);

  Ok(())
}

#[test]
fn takes_each_corners_first_reference_entry_and_its_surfaces_shader_and_texture() -> XrfResult {
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  let at = |position: [f32; 3], uv: [f32; 2]| with_uv(unweighted(corner(position, UP)), uv);
  let compiled: ObjectCompile = compile(&static_object(assemble_static(&[surface(vec![[
    at(a, [0.0, 0.0]),
    at(b, [0.25, 0.0]),
    at(c, [0.0, 0.75]),
  ]])])?))?;
  let model: DetailModel = read(&compiled)?;

  assert_eq!(
    (model.shader.as_str(), model.texture.as_str()),
    ("models\\model", "wood\\wood_plank6")
  );
  assert_eq!(
    model.vertices.iter().map(|it| (it.u, it.v)).collect::<Vec<_>>(),
    [(0.0, 0.0), (0.25, 0.0), (0.0, 0.75)]
  );
  assert_eq!((model.flags, model.min_scale, model.max_scale), (0, 0.5, 2.0));

  Ok(())
}

#[test]
fn refuses_an_object_of_other_than_one_surface() -> XrfResult {
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  let at = |position: [f32; 3]| unweighted(corner(position, UP));
  let mut other = surface(vec![[at(a), at(c), at(b)]]);

  other.texture = String::from("wood\\wood_plank7");

  let object: EditableObjectFile = static_object(assemble_static(&[surface(vec![[at(a), at(b), at(c)]]), other])?);
  let error: String = compile(&object).expect_err("Expect two surfaces refused").to_string();

  assert!(error.contains("one surface, not 2"), "{error}");

  Ok(())
}
