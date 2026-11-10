use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface};
use xrf_error::XrfResult;
use xrf_level::LevelVertex;
use xrf_math::{Vector2d, Vector3d};

use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A level vertex at `position` whose normal is `normal` stored as xrLC stores it, a byte a component, `(n + 1) *
/// 127.5` truncated.
fn vertex(position: [f32; 3], normal: [f32; 3]) -> LevelVertex {
  let byte = |component: f32| ((component + 1.0) * 127.5) as u8;
  let (normal, _) = LevelVertex::decode_direction([byte(normal[2]), byte(normal[1]), byte(normal[0]), 0]);

  LevelVertex {
    position: Vector3d::from(position),
    normal,
    tangent: None,
    binormal: None,
    texture_coordinate: Vector2d::new(position[0], position[2]),
    lightmap_coordinate: None,
    color: None,
    hemi: 0,
  }
}

/// A visual drawn with `texture` of `vertices`, its triangles `indices`.
fn visual(texture: &str, vertices: Vec<LevelVertex>, indices: Vec<u16>) -> LevelVisualGeometry {
  LevelVisualGeometry {
    texture: String::from(texture),
    shader: String::from("def_shaders\\def_vertex"),
    dress: SurfaceDress::default(),
    vertices,
    indices,
  }
}

fn decompile(geometry: &[LevelVisualGeometry]) -> XrfResult<EditableObjectFile> {
  Ok(ObjectDecompiler::decompile_level_geometry(geometry)?.object)
}

fn mesh(object: &EditableObjectFile) -> &EditableMesh {
  &object.meshes.as_ref().expect("Expect a mesh")[0]
}

const UP: [f32; 3] = [0.0, 1.0, 0.0];
const DOWN: [f32; 3] = [0.0, -1.0, 0.0];

#[test]
fn folds_a_mirror_one_quantisation_step_off_into_a_two_sided_face() -> XrfResult {
  // Up stores as (127, 255, 127), down as (127, 0, 127): each zero truncates to the step below it, so the mirror's
  // normal turned around stands one step from the face's, as every mirrored corner of the vanilla levels does.
  let [a, b, c] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
  let object: EditableObjectFile = decompile(&[visual(
    "wood\\wood_plank6",
    vec![
      vertex(a, UP),
      vertex(b, UP),
      vertex(c, UP),
      vertex(a, DOWN),
      vertex(b, DOWN),
      vertex(c, DOWN),
    ],
    vec![0, 1, 2, 5, 4, 3],
  )])?;
  let surfaces: &[EditableSurface] = object.surfaces.as_deref().expect("Expect surfaces");

  assert_eq!(mesh(&object).faces.len(), 1);
  assert!(surfaces.len() == 1 && surfaces[0].is_two_sided());

  Ok(())
}

#[test]
fn smooths_a_seam_one_step_apart_and_keeps_a_hard_edge_hard() -> XrfResult {
  // A quad split along `a c` at a seam, each half its own vertices: one smoothed normal stored a step apart on either
  // side, or two faces meeting at a hard edge.
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
  let seam = |other: [f32; 3]| {
    visual(
      "wood\\wood_plank6",
      vec![
        vertex(a, UP),
        vertex(b, UP),
        vertex(c, UP),
        vertex(a, other),
        vertex(c, other),
        vertex(d, other),
      ],
      vec![0, 1, 2, 3, 4, 5],
    )
  };
  // One step off up in x and z, as two corners of one smoothed normal can come back.
  let step_off: [f32; 3] = [1.0 / 127.5, 1.0, 1.0 / 127.5];
  let soft: EditableObjectFile = decompile(&[seam(step_off)])?;

  assert_eq!(mesh(&soft).points.len(), 4);
  assert_eq!(mesh(&soft).smoothing_groups, Some(vec![0b011, 0b110]));

  let hard: EditableObjectFile = decompile(&[seam([0.0, 0.0, 1.0])])?;

  assert_eq!(
    mesh(&hard).smoothing_groups,
    Some(vec![
      EditableMesh::FLAT_FACE_SMOOTHING,
      EditableMesh::FLAT_FACE_SMOOTHING
    ])
  );

  Ok(())
}

#[test]
fn takes_the_visuals_of_one_texture_and_shader_as_one_surface_welded_across_them() -> XrfResult {
  // Two halves of a quad in two visuals of one texture, and a third visual of another texture beside them.
  let [a, b, c, d, e] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [2.0, 0.0, 0.0],
  ];
  let object: EditableObjectFile = decompile(&[
    visual(
      "wood\\wood_plank6",
      vec![vertex(a, UP), vertex(b, UP), vertex(c, UP)],
      vec![0, 1, 2],
    ),
    visual(
      "ston\\ston_stena",
      vec![vertex(b, UP), vertex(e, UP), vertex(c, UP)],
      vec![0, 1, 2],
    ),
    visual(
      "wood\\wood_plank6",
      vec![vertex(a, UP), vertex(c, UP), vertex(d, UP)],
      vec![0, 1, 2],
    ),
  ])?;
  let mesh: &EditableMesh = mesh(&object);

  assert_eq!(
    object
      .surfaces
      .iter()
      .flatten()
      .map(|surface| surface.texture.as_str())
      .collect::<Vec<_>>(),
    ["wood\\wood_plank6", "ston\\ston_stena"]
  );
  assert_eq!(
    mesh
      .surface_faces
      .iter()
      .map(|faces| faces.faces.len())
      .collect::<Vec<_>>(),
    [2, 1]
  );
  assert_eq!(mesh.points.len(), 5);
  assert_eq!(object.flags, 0);
  assert!(object.bones.is_none());

  Ok(())
}

#[test]
fn refuses_an_index_past_a_visuals_vertices() {
  let error: String = ObjectDecompiler::decompile_level_geometry(&[visual(
    "wood\\wood_plank6",
    vec![vertex([0.0; 3], UP)],
    vec![0, 0, 7],
  )])
  .expect_err("Expect the visual refused")
  .to_string();

  assert!(error.contains("Index 7"), "{error}");
}
