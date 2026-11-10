use std::collections::BTreeMap;

use xrf_chunk::{ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_editable_level::{EditableObjectTransform, EditableSectorItem};
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::{
  LevelCformFace, LevelCformGeometry, LevelFile, LevelGeomFile, LevelGeomSource, LevelHeaderChunk, LevelPortal,
  LevelPortalsChunk, LevelSector, LevelSectorsChunk, LevelShaderEntry, LevelShaderReference, LevelShadersChunk,
  LevelVisual, LevelVisualsChunk, LevelWallmark, LevelWallmarkSlot, LevelWallmarkVertex, LevelWallmarksFile,
};
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{OgfGeometryContainerChunk, OgfHeaderChunk};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::level::tests::GUID;
use crate::{
  LevelCollisionIndex, LevelCompilerShaderReason, LevelDecalIndex, LevelDecompileContext, LevelGeometryCensus,
  LevelGeometryConversion, LevelGeometryDecompile, LevelGeometrySources, LevelShaderPairings, LevelSurfaceCensus,
  LevelSurfaceInference, SurfaceDress,
};

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
  decompile_with(level, vec![drawable(0), drawable(1), hierarchy(vec![0])], None, None)
}

/// The two squares' geometry, drawn by `visuals` as `level` arranges them in sectors, with `collision` and `inference`
/// where given.
fn decompile_with(
  level: &LevelFile,
  visuals: Vec<LevelVisual>,
  collision: Option<&mut LevelCollisionIndex>,
  inference: Option<&mut LevelSurfaceInference>,
) -> XrfResult<LevelGeometryDecompile> {
  LevelGeometryConversion::decompile::<XRayByteOrder, _>(
    LevelGeometrySources {
      name: "test",
      level,
      visuals: &LevelVisualsChunk { visuals },
      geom: &geometry(&[square(-4.0), square(4.0)])?,
      collision,
      hom: None,
      som: None,
      decals: None,
      spawn: None,
      breakable_parts: &BTreeMap::new(),
    },
    inference,
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

  let decompiled: LevelGeometryDecompile = decompile_with(
    &level,
    vec![drawable(0), drawable(1), hierarchy(vec![0]), hierarchy(Vec::new())],
    None,
    None,
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

const STONE: u16 = 3;
const EARTH: u16 = 5;

/// The two squares' collision form: the west one's own triangles as faces of stone, where `with_earth` a larger face of
/// earth beneath the east one, which only a ray finds, and where `with_wall` a wall of stone in the second sector no
/// render triangle draws.
fn collision(with_earth: bool, with_wall: bool) -> XrfResult<LevelCollisionIndex> {
  let face = |vertices: [u32; 3], material: u16, sector: u16| LevelCformFace {
    vertices,
    material,
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    sector,
  };
  let [a, b, c, d] = square(-4.0).map(|[x, y, z]| Vector3d::new(x, y, z));
  let mut vertices: Vec<Vector3d> = vec![a, b, c, d];
  let mut faces: Vec<LevelCformFace> = vec![face([0, 2, 1], STONE, 0), face([0, 3, 2], STONE, 0)];

  if with_earth {
    vertices.extend([
      Vector3d::new(2.0, 0.0, -3.0),
      Vector3d::new(2.0, 0.0, 6.0),
      Vector3d::new(11.0, 0.0, -3.0),
    ]);
    faces.push(face([4, 5, 6], EARTH, 1));
  }

  if with_wall {
    let base: u32 = vertices.len() as u32;

    vertices.extend([
      Vector3d::new(20.0, 0.0, 0.0),
      Vector3d::new(20.0, 3.0, 0.0),
      Vector3d::new(24.0, 0.0, 0.0),
    ]);
    faces.push(face([base, base + 1, base + 2], STONE, 1));
  }

  Ok(LevelCollisionIndex::new(LevelCformGeometry::new(vertices, faces)?))
}

/// What names the materials and offers compiler shaders.
fn inference() -> LevelSurfaceInference {
  let shader = |name: &str, flags: u32| ShaderCompilerShader {
    name: String::from(name),
    name_trailing: Vec::new(),
    flags,
    vertex_translucency: 0.0,
    vertex_ambient: 0.0,
    lightmap_density: 1.0,
  };

  LevelSurfaceInference::new(
    GameMtlFile::new(
      vec![
        GameMtlMaterial::new(u32::from(STONE), "materials\\stone"),
        GameMtlMaterial::new(u32::from(EARTH), "materials\\earth"),
      ],
      Vec::new(),
    ),
    ShaderCompilerFile {
      shaders: vec![
        shader("default", 0b0011),
        shader("def_shaders\\def_vertex", 0b1011),
        shader("def_shaders\\def_vertex_ghost", 0b1010),
      ],
    },
    LevelShaderPairings::default(),
  )
}

/// Each shell's surfaces, as compiler shader and game material.
fn dress_of(decompiled: &LevelGeometryDecompile) -> Vec<Vec<(String, String)>> {
  decompiled
    .objects
    .iter()
    .map(|object| {
      object
        .decompile
        .object
        .surfaces
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|surface| (surface.compiler_shader.clone(), surface.game_material.clone()))
        .collect()
    })
    .collect()
}

fn decompile_dressed(
  collision: &mut LevelCollisionIndex,
  inference: &mut LevelSurfaceInference,
) -> XrfResult<LevelGeometryDecompile> {
  decompile_with(
    &level((0, 1)),
    vec![drawable(0), drawable(1), hierarchy(vec![0])],
    Some(collision),
    Some(inference),
  )
}

#[test]
fn dresses_surfaces_from_the_collision_faces_they_lie_on() -> XrfResult {
  let mut inference: LevelSurfaceInference = inference();
  let decompiled: LevelGeometryDecompile = decompile_dressed(&mut collision(true, false)?, &mut inference)?;
  let vertex = String::from("def_shaders\\def_vertex");

  assert_eq!(
    dress_of(&decompiled),
    [
      vec![(vertex.clone(), String::from("materials\\stone"))],
      vec![(vertex, String::from("materials\\earth"))]
    ],
    "vertex lit, as their vertices keep no lightmap coordinates, and colliding"
  );

  let census: LevelSurfaceCensus = inference.into_census();

  assert_eq!(
    (
      census.exact_triangles,
      census.traced_triangles,
      census.uncollided_triangles
    ),
    (2, 2, 0)
  );
  assert_eq!(
    census.compiler_shaders.len(),
    1,
    "one engine shader, one kind of evidence"
  );
  assert_eq!(
    census.compiler_shaders[0].reason,
    LevelCompilerShaderReason::CommonestFit
  );

  Ok(())
}

#[test]
fn gives_a_surface_on_no_collision_face_a_compiler_shader_of_no_collision() -> XrfResult {
  let mut inference: LevelSurfaceInference = inference();
  let decompiled: LevelGeometryDecompile = decompile_dressed(&mut collision(false, false)?, &mut inference)?;

  assert_eq!(
    dress_of(&decompiled)[1],
    [(
      String::from("def_shaders\\def_vertex_ghost"),
      String::from(SurfaceDress::DEFAULT_SETTING)
    )]
  );
  assert_eq!(inference.into_census().uncollided_triangles, 2);

  Ok(())
}

#[test]
fn gathers_collision_no_render_triangle_draws_into_its_own_object_by_sector() -> XrfResult {
  let decompiled: LevelGeometryDecompile = decompile_dressed(&mut collision(true, true)?, &mut inference())?;
  let collision = decompiled.objects.last().expect("the collision object is written last");
  let meshes = collision.decompile.object.meshes.as_deref().unwrap_or_default();

  assert_eq!(collision.reference, "levels\\test\\test_collision");
  assert_eq!(collision.source, "level.cform", "its sidecar names the collision form");
  assert_eq!(decompiled.objects[0].source, "level.geom");
  assert_eq!(
    meshes
      .iter()
      .map(|mesh| (mesh.name.as_str(), mesh.faces.len()))
      .collect::<Vec<_>>(),
    [("collision_1", 2)],
    "the sector's lone collision face, halved for the Level Editor"
  );
  assert_eq!(
    dress_of(&decompiled).last().cloned().unwrap_or_default(),
    [(
      String::from(LevelGeometryConversion::COLLISION_COMPILER_SHADER),
      String::from("materials\\stone")
    )]
  );
  assert_eq!(
    decompiled.sectors.objects[1].items.last(),
    Some(&EditableSectorItem {
      object: collision.reference.clone(),
      mesh: String::from("collision_1"),
    }),
    "the sector its faces lie in names its mesh"
  );
  assert_eq!(
    (
      decompiled.census.collision_faces,
      decompiled.census.collision_only_faces
    ),
    (4, 1)
  );
  assert!(
    decompiled
      .scene_objects
      .objects
      .iter()
      .any(|placed| placed.reference == collision.reference)
  );

  Ok(())
}

#[test]
fn drops_the_static_copies_of_wall_marks() -> XrfResult {
  let corner = |[x, y, z]: [f32; 3]| LevelWallmarkVertex {
    position: Vector3d::new(x, y, z),
    color: 0,
    u: 0.0,
    v: 0.0,
  };
  let east: [[f32; 3]; 4] = square(4.0);
  // The Build adds each decal to the static faces too: here one triangle of the east square, a slot of its texture.
  let wallmarks: LevelWallmarksFile = LevelWallmarksFile {
    slots: vec![LevelWallmarkSlot {
      shader: String::from("default"),
      texture: String::from("floor\\floor_tile"),
      marks: vec![LevelWallmark {
        center: Vector3d::new(4.0, 0.0, 0.0),
        radius: 1.5,
        vertices: vec![corner(east[0]), corner(east[1]), corner(east[2])],
      }],
    }],
  };
  let decompiled: LevelGeometryDecompile = LevelGeometryConversion::decompile::<XRayByteOrder, _>(
    LevelGeometrySources {
      name: "test",
      level: &level((0, 1)),
      visuals: &LevelVisualsChunk {
        visuals: vec![drawable(0), drawable(1), hierarchy(vec![0])],
      },
      geom: &geometry(&[square(-4.0), east])?,
      collision: None,
      hom: None,
      som: None,
      decals: Some(&LevelDecalIndex::new(&wallmarks)),
      spawn: None,
      breakable_parts: &BTreeMap::new(),
    },
    None,
    &mut LevelDecompileContext::new(GUID),
  )?;

  assert_eq!(
    (
      decompiled.census.static_triangles,
      decompiled.census.faces,
      decompiled.census.decal_triangles
    ),
    (4, 3, 1),
    "the decal's copy is counted, not written"
  );

  Ok(())
}

#[test]
fn takes_a_triangle_xrlc_measures_no_area_in_for_a_needle() {
  // One of the faces of `labx8`'s compiled geometry xrLC refused: 11 m long, about 2 mm across.
  assert!(LevelGeometryConversion::is_needle(&[
    Vector3d::new(-75.476135, 2.162382, 99.757_22),
    Vector3d::new(-75.4795, -1.505456, 99.757_22),
    Vector3d::new(-75.4795, -8.841132, 99.757_22),
  ]));
  assert!(LevelGeometryConversion::is_needle(&[Vector3d::ZERO; 3]));
  // A sliver a centimetre across holds area enough.
  assert!(!LevelGeometryConversion::is_needle(&[
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(5.0, 0.01, 0.0),
    Vector3d::new(10.0, 0.0, 0.0),
  ]));
}

#[test]
fn refuses_a_portal_joining_a_sector_the_level_does_not_hold() -> XrfResult {
  let error = decompile(&level((0, 5))).expect_err("a portal past the sectors is refused");

  assert!(error.to_string().contains("joins sector 5"), "{error}");

  Ok(())
}
