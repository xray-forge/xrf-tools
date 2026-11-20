//! Editable objects built in code, as a level's visuals decompile into one: a surface a run of bare triangles.

use xrf_editable_object::EditableObjectFile;
use xrf_level::LevelVertex;
use xrf_math::{Vector2d, Vector3d};
use xrf_rawdata::{LevelVisualGeometry, ObjectCompileSource, ObjectDecompiler, SurfaceDress};

/// A surface drawn with `shader` and `texture`, of `triangles` each facing the side its corners wind towards.
pub(super) fn surface(shader: &str, texture: &str, triangles: &[[Vector3d; 3]]) -> LevelVisualGeometry {
  let vertices: Vec<LevelVertex> = triangles
    .iter()
    .flat_map(|[a, b, c]| {
      let cross: Vector3d = (b - a).cross(&(c - a));
      let normal: Vector3d = &cross / cross.length();

      [(a, 0.0, 0.0), (b, 1.0, 0.0), (c, 0.0, 1.0)].map(|(position, u, v)| LevelVertex {
        position: *position,
        normal,
        tangent: None,
        binormal: None,
        texture_coordinate: Vector2d::new(u, v),
        lightmap_coordinate: None,
        color: None,
        hemi: 0,
      })
    })
    .collect();

  LevelVisualGeometry {
    texture: String::from(texture),
    shader: String::from(shader),
    dress: SurfaceDress {
      compiler_shader: String::from(SurfaceDress::DEFAULT_SETTING),
      game_material: String::from(SurfaceDress::DEFAULT_SETTING),
    },
    indices: (0..vertices.len() as u16).collect(),
    vertices,
  }
}

/// A flat quad on the ground spanning `0..width` along `x` and `0..depth` along `z`, as two triangles facing up.
pub(super) fn quad(width: f32, depth: f32) -> Vec<[Vector3d; 3]> {
  let [a, b, c, d] = [
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(width, 0.0, 0.0),
    Vector3d::new(width, 0.0, depth),
    Vector3d::new(0.0, 0.0, depth),
  ];

  vec![[a, c, b], [a, d, c]]
}

/// A static object of these surfaces, each its own.
pub(super) fn object_of(surfaces: &[LevelVisualGeometry]) -> EditableObjectFile {
  ObjectDecompiler::decompile_level_geometry(surfaces)
    .expect("the surfaces decompile into an object")
    .object
}

/// What a test compiles an object as.
pub(super) fn compile_source() -> ObjectCompileSource {
  ObjectCompileSource {
    source_file: String::from("test.object"),
    convertor: String::from("xrf-object-visual tests"),
    built_at: 0,
  }
}
