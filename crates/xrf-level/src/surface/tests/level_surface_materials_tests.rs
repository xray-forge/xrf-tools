use std::collections::BTreeMap;

use xrf_chunk::{InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_ogf::{OgfBox, OgfGeometryContainerChunk, OgfHeaderChunk, OgfSphere};

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_geometry::LevelCformGeometry;
use crate::cform::level_cform_tracer::LevelCformTracer;
use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::level_geom_source::LevelGeomSource;
use crate::level::visuals::level_visual::LevelVisual;
use crate::level::visuals::level_visuals_chunk::LevelVisualsChunk;
use crate::surface::level_surface_material::LevelSurfaceMaterial;
use crate::surface::level_surface_materials::LevelSurfaceMaterials;
use crate::tests::fixtures::chunk;

const STONE: u16 = 3;
const EARTH: u16 = 7;

/// A floor at the ground, stone west of the origin and earth east of it, each a square eight metres across.
fn floor() -> XrfResult<LevelCformGeometry> {
  let face = |vertices: [u32; 3], material: u16| LevelCformFace {
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    material,
    sector: 0,
    vertices,
  };

  LevelCformGeometry::new(
    vec![
      Vector3d::new(-8.0, 0.0, -4.0),
      Vector3d::new(0.0, 0.0, -4.0),
      Vector3d::new(0.0, 0.0, 4.0),
      Vector3d::new(-8.0, 0.0, 4.0),
      Vector3d::new(8.0, 0.0, -4.0),
      Vector3d::new(8.0, 0.0, 4.0),
    ],
    vec![
      face([0, 2, 1], STONE),
      face([0, 3, 2], STONE),
      face([1, 2, 5], EARTH),
      face([1, 5, 4], EARTH),
    ],
  )
}

/// A square of render geometry two metres across, at `height`, its corners centred on `x`.
fn square(x: f32, height: f32) -> [[f32; 3]; 4] {
  [
    [x - 1.0, height, -1.0],
    [x + 1.0, height, -1.0],
    [x + 1.0, height, 1.0],
    [x - 1.0, height, 1.0],
  ]
}

/// A `level.geom` of positions alone, one buffer of the squares' corners and one of two triangles each.
fn geometry(squares: &[[[f32; 3]; 4]]) -> XrfResult<Vec<u8>> {
  let mut declaration: Vec<u8> = Vec::new();

  // POSITION0:FLOAT3, then `D3DDECL_END`.
  declaration.extend_from_slice(&0u16.to_le_bytes());
  declaration.extend_from_slice(&0u16.to_le_bytes());
  declaration.extend_from_slice(&[2, 0, 0, 0]);
  declaration.extend_from_slice(&LevelGeomVertexElement::TERMINATOR_STREAM.to_le_bytes());
  declaration.extend_from_slice(&0u16.to_le_bytes());
  declaration.extend_from_slice(&[17, 0, 0, 0]);

  let mut vertices: Vec<u8> = 1u32.to_le_bytes().to_vec();

  vertices.extend_from_slice(&declaration);
  vertices.extend_from_slice(&(squares.len() as u32 * 4).to_le_bytes());

  for corner in squares.iter().flatten().flatten() {
    vertices.extend_from_slice(&corner.to_le_bytes());
  }

  let mut indices: Vec<u8> = 1u32.to_le_bytes().to_vec();

  indices.extend_from_slice(&(squares.len() as u32 * 6).to_le_bytes());

  for _ in squares {
    for index in [0u16, 1, 2, 0, 2, 3] {
      indices.extend_from_slice(&index.to_le_bytes());
    }
  }

  let mut bytes: Vec<u8> = chunk(LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID, &vertices)?;

  bytes.extend(chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, &indices)?);

  Ok(bytes)
}

/// The visual drawing the `index`th square with the shader table entry `shader`.
fn visual(index: u32, shader: u16) -> LevelVisual {
  LevelVisual {
    header: OgfHeaderChunk {
      version: 4,
      model_type: 0,
      shader_id: shader,
      bounding_box: OgfBox {
        min: Vector3d::new(0.0, 0.0, 0.0),
        max: Vector3d::new(0.0, 0.0, 0.0),
      },
      bounding_sphere: OgfSphere {
        position: Vector3d::new(0.0, 0.0, 0.0),
        radius: 0.0,
      },
    },
    texture: None,
    geometry: Some(OgfGeometryContainerChunk {
      vertex_buffer_id: 0,
      vertex_base: index * 4,
      vertex_count: 4,
      index_buffer_id: 0,
      index_base: index * 6,
      index_count: 6,
    }),
    fastpath: None,
    children: Vec::new(),
    tree: None,
    lod: None,
    swi: None,
    swi_container: None,
  }
}

#[test]
fn tallies_each_surface_by_the_collision_faces_beneath_it() -> XrfResult {
  // Stone under the first, earth under the second, nothing within reach of the third, which floats a metre up.
  let source: LevelGeomSource<InMemoryChunkDataSource> =
    LevelGeomSource::open_from_bytes::<XRayByteOrder>(geometry(&[
      square(-4.0, 0.0),
      square(4.0, 0.0),
      square(4.0, 1.0),
    ])?)?;
  let visuals: LevelVisualsChunk = LevelVisualsChunk {
    visuals: vec![visual(0, 1), visual(1, 2), visual(2, 2)],
  };
  let survey: LevelSurfaceMaterials =
    LevelSurfaceMaterials::survey::<XRayByteOrder, _>(&visuals, &source, &LevelCformTracer::new(&floor()?));

  let stone: &LevelSurfaceMaterial = &survey.surfaces[&1];
  let earth: &LevelSurfaceMaterial = &survey.surfaces[&2];

  assert_eq!(survey.unread_visuals, 0);
  assert_eq!((stone.area, stone.uncovered_area), (4.0, 0.0));
  assert_eq!(stone.materials, BTreeMap::from([(STONE, 4.0)]));
  assert_eq!(
    (earth.area, earth.uncovered_area),
    (8.0, 4.0),
    "the floating square adds its area, on nothing"
  );
  assert_eq!(earth.materials, BTreeMap::from([(EARTH, 4.0)]));

  Ok(())
}
