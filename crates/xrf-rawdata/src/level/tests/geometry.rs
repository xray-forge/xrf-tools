use xrf_chunk::{ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_editable_level::{EditableObjectTransform, EditableSectorItem};
use xrf_error::XrfResult;
use xrf_level::{
  LevelFile, LevelGeomFile, LevelGeomSource, LevelHeaderChunk, LevelPortal, LevelPortalsChunk, LevelSector,
  LevelSectorsChunk, LevelShaderEntry, LevelShaderReference, LevelShadersChunk, LevelVisual, LevelVisualsChunk,
};
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{OgfGeometryContainerChunk, OgfHeaderChunk};

use crate::level::tests::GUID;
use crate::{LevelDecompileContext, LevelGeometryCensus, LevelGeometryConversion, LevelGeometryDecompile};

/// A square of render geometry two metres across on the ground, centred on `x`.
fn square(x: f32) -> [[f32; 3]; 4] {
  [
    [x - 1.0, 0.0, -1.0],
    [x - 1.0, 0.0, 1.0],
    [x + 1.0, 0.0, 1.0],
    [x + 1.0, 0.0, -1.0],
  ]
}

fn chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// A `level.geom` of positions alone: one buffer of the squares' corners, and one of two triangles a square.
fn geometry(squares: &[[[f32; 3]; 4]]) -> XrfResult<LevelGeomSource<InMemoryChunkDataSource>> {
  let mut vertices: Vec<u8> = 1u32.to_le_bytes().to_vec();

  // POSITION0:FLOAT3, then `D3DDECL_END`.
  vertices.extend_from_slice(&[0, 0, 0, 0, 2, 0, 0, 0]);
  vertices.extend_from_slice(&[0xFF, 0, 0, 0, 17, 0, 0, 0]);
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

  LevelGeomSource::open_from_bytes::<XRayByteOrder>(bytes)
}

fn header(model_type: u8) -> OgfHeaderChunk {
  OgfHeaderChunk {
    version: 4,
    model_type,
    shader_id: 1,
    bounding_box: Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::ZERO,
    },
    bounding_sphere: Sphere {
      center: Vector3d::ZERO,
      radius: 0.0,
    },
  }
}

/// The visual drawing the `index`th square.
fn drawable(index: u32) -> LevelVisual {
  LevelVisual {
    header: header(0),
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

fn hierarchy(children: Vec<u32>) -> LevelVisual {
  LevelVisual {
    geometry: None,
    children,
    header: header(1),
    ..drawable(0)
  }
}

/// Two sectors joined by a portal: the first a hierarchy over the square west of the origin, the second the square
/// east of it as its own root.
fn level(portal_sectors: (u16, u16)) -> LevelFile {
  LevelFile {
    header: LevelHeaderChunk {
      xrlc_version: 14,
      xrlc_quality: 0,
    },
    shaders: Some(LevelShadersChunk {
      entries: vec![
        LevelShaderEntry::Empty,
        LevelShaderEntry::Reference(LevelShaderReference {
          shader: String::from("default"),
          textures: vec![String::from("floor\\floor_tile")],
        }),
      ],
    }),
    sectors: Some(LevelSectorsChunk {
      sectors: vec![
        LevelSector {
          portals: vec![0],
          root: 2,
        },
        LevelSector {
          portals: vec![0],
          root: 1,
        },
      ],
    }),
    portals: Some(LevelPortalsChunk {
      portals: vec![LevelPortal {
        sector_front: portal_sectors.0,
        sector_back: portal_sectors.1,
        vertices: vec![
          Vector3d::new(0.0, 0.0, -1.0),
          Vector3d::new(0.0, 2.0, -1.0),
          Vector3d::new(0.0, 2.0, 1.0),
          Vector3d::new(0.0, 0.0, 1.0),
          Vector3d::ZERO,
          Vector3d::ZERO,
        ],
        vertex_count: 4,
      }],
    }),
    lights: None,
    glows: None,
  }
}

fn decompile(level: &LevelFile) -> XrfResult<LevelGeometryDecompile> {
  LevelGeometryConversion::decompile::<XRayByteOrder, _>(
    "test",
    level,
    &LevelVisualsChunk {
      visuals: vec![drawable(0), drawable(1), hierarchy(vec![0])],
    },
    &geometry(&[square(-4.0), square(4.0)])?,
    &mut LevelDecompileContext::new(GUID),
  )
}

#[test]
fn decompiles_each_sector_into_a_shell_with_its_sector_and_portals() -> XrfResult {
  let decompiled: LevelGeometryDecompile = decompile(&level((0, 1)))?;

  assert_eq!(
    decompiled
      .objects
      .iter()
      .map(|object| object.reference.as_str())
      .collect::<Vec<_>>(),
    ["levels\\test\\test_sector_0", "levels\\test\\test_sector_1"]
  );
  assert_eq!(
    decompiled.census,
    LevelGeometryCensus {
      static_triangles: 4,
      faces: 4,
      ..LevelGeometryCensus::default()
    }
  );

  let points = &decompiled.objects[1]
    .decompile
    .object
    .meshes
    .as_deref()
    .unwrap_or_default()[0]
    .points;

  assert!(
    points.contains(&Vector3d::new(5.0, 0.0, 1.0)),
    "a shell keeps the level's own positions"
  );

  for (placed, object) in decompiled.scene_objects.objects.iter().zip(&decompiled.objects) {
    assert_eq!(placed.object.name, object.reference);
    assert_eq!(placed.reference, object.reference);
    assert_eq!(placed.object.transform, EditableObjectTransform::default());
  }

  let sectors = &decompiled.sectors.objects;

  assert_eq!(
    sectors.iter().map(|it| it.object.name.as_str()).collect::<Vec<_>>(),
    ["sector_0000", "sector_0001"]
  );
  assert_eq!(
    sectors[1].items,
    [EditableSectorItem {
      object: String::from("levels\\test\\test_sector_1"),
      mesh: decompiled.objects[1]
        .decompile
        .object
        .meshes
        .as_deref()
        .unwrap_or_default()[0]
        .name
        .clone(),
    }]
  );
  assert!(sectors.iter().all(|sector| !sector.is_default));

  let portal = &decompiled.portals.objects[0];

  assert_eq!(
    (portal.sector_front.as_str(), portal.sector_back.as_str()),
    ("sector_0000", "sector_0001")
  );
  assert_eq!(portal.vertices.len(), 4, "only the corners the portal spans");

  Ok(())
}

#[test]
fn leaves_sectors_and_portals_out_where_a_sector_draws_no_static_geometry() -> XrfResult {
  let mut level: LevelFile = level((0, 1));

  // The second sector's root is a hierarchy over nothing, as a sector of trees alone draws no static geometry.
  level.sectors.as_mut().expect("the level holds sectors").sectors[1].root = 3;

  let decompiled: LevelGeometryDecompile = LevelGeometryConversion::decompile::<XRayByteOrder, _>(
    "test",
    &level,
    &LevelVisualsChunk {
      visuals: vec![drawable(0), drawable(1), hierarchy(vec![0]), hierarchy(Vec::new())],
    },
    &geometry(&[square(-4.0), square(4.0)])?,
    &mut LevelDecompileContext::new(GUID),
  )?;

  assert_eq!(
    decompiled.objects.len(),
    1,
    "the drawing sector's shell is still written"
  );
  assert_eq!(decompiled.census.empty_sectors, 1);
  assert!(decompiled.sectors.objects.is_empty());
  assert!(decompiled.portals.objects.is_empty());
  assert_eq!(decompiled.refused.len(), 1);
  assert_eq!(decompiled.refused[0].object, "sector 1");

  Ok(())
}

#[test]
fn refuses_a_portal_joining_a_sector_the_level_does_not_hold() -> XrfResult {
  let error = decompile(&level((0, 5))).expect_err("a portal past the sectors is refused");

  assert!(error.to_string().contains("joins sector 5"), "{error}");

  Ok(())
}
