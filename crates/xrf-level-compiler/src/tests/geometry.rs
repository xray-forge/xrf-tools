use xrf_error::XrfResult;
use xrf_level::{
  BuildPrjFace, BuildPrjFile, BuildPrjMaterial, BuildPrjMuModel, BuildPrjMuReference, BuildPrjName, BuildPrjParams,
  BuildPrjShader, BuildPrjTexture, LevelCformFace, LevelCformGeometry,
};
use xrf_math::{Aabb, Matrix4x4, Vector2d, Vector3d};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::geometry::geometry_collector::GeometryCollector;
use crate::{CompilerShaders, GeometryCollision, GeometryFace, GeometryMesh, GeometryNormals, GeometryOptimization};

/// The edge flags of a front face with every edge soft.
const SOFT: u32 = 0;

/// A project of `vertices` and `faces` (corners, then edge flags), each face of material 0, mapped across the texture.
fn project(vertices: &[[f32; 3]], faces: &[([u32; 3], u32)]) -> XrfResult<BuildPrjFile> {
  let material = |compiler_shader: u16| BuildPrjMaterial {
    texture: 0,
    shader: 0,
    compiler_shader,
    sector: 0,
    reserved: 0,
    area_padding: 0,
    internal_max_area: 0,
  };

  Ok(BuildPrjFile {
    version: BuildPrjFile::CURRENT_VERSION,
    params: BuildPrjParams {
      smooth_angle: 75.0,
      weld_distance: 0.005,
      lightmap_pixels_per_meter: 0.0,
      lightmap_jitter_samples: 0,
      lightmap_rms_zero: 0,
      lightmap_rms: 0,
      quality: 0,
      quality_reserved: 0,
      reserved: [0.0; 6],
    },
    vertices: vertices.iter().map(|vertex| Vector3d::from(*vertex)).collect(),
    faces: faces
      .iter()
      .map(|(corners, _)| BuildPrjFace {
        vertices: *corners,
        texture_coordinates: [
          Vector2d::new(0.0, 0.0),
          Vector2d::new(1.0, 0.0),
          Vector2d::new(0.0, 1.0),
        ],
        material: 0,
        material_padding: 0,
        game_material: 0,
      })
      .collect(),
    smoothing_groups: Some(faces.iter().map(|(_, flags)| *flags).collect()),
    // The second material has no compiler shader, as a glow's; the third is drawn and does not collide.
    materials: vec![material(0), material(u16::MAX), material(1)],
    render_shaders: vec![BuildPrjShader {
      name: BuildPrjName::new("default", BuildPrjName::LONG_SIZE)?,
    }],
    compiler_shaders: vec![
      BuildPrjShader {
        name: BuildPrjName::new("default", BuildPrjName::LONG_SIZE)?,
      },
      BuildPrjShader {
        name: BuildPrjName::new("ghost", BuildPrjName::LONG_SIZE)?,
      },
    ],
    textures: vec![BuildPrjTexture {
      name: BuildPrjName::new("ground", BuildPrjName::LONG_SIZE)?,
      width: 0,
      height: 0,
      has_alpha: 0,
      surface: 0,
    }],
    glows: Vec::new(),
    portals: Vec::new(),
    light_controls: Vec::new(),
    static_lights: Vec::new(),
    dynamic_lights: Vec::new(),
    lods: Vec::new(),
    mu_models: Vec::new(),
    mu_references: Vec::new(),
  })
}

/// The compiler shader library the project's two compiler shaders resolve in: `default` collides and is drawn,
/// `ghost` is only drawn.
fn library() -> ShaderCompilerFile {
  let shader = |name: &str, flags: u32| ShaderCompilerShader {
    name: String::from(name),
    name_trailing: vec![0; ShaderCompilerShader::NAME_SIZE - name.len()],
    flags,
    vertex_translucency: 0.5,
    vertex_ambient: 0.0,
    lightmap_density: 1.0,
  };

  ShaderCompilerFile {
    shaders: vec![
      shader(
        "default",
        ShaderCompilerShader::COLLISION | ShaderCompilerShader::RENDERING,
      ),
      shader("ghost", ShaderCompilerShader::RENDERING),
    ],
  }
}

/// A unit square on the floor, `0 1 2 3` round its edge, as two faces over the diagonal `0 2` with the flags given.
fn square(first: u32, second: u32) -> XrfResult<BuildPrjFile> {
  project(
    &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    &[([0, 2, 1], first), ([0, 3, 2], second)],
  )
}

fn error_of<T>(result: XrfResult<T>) -> String {
  result.err().map(|error| error.to_string()).unwrap_or_default()
}

#[test]
fn loads_a_project_and_refuses_one_xrlc_would_stop_on_or_read_past() -> XrfResult {
  let mesh: GeometryMesh = GeometryMesh::load(&square(SOFT, SOFT)?)?;

  assert_eq!((mesh.vertices.len(), mesh.faces.len()), (4, 2));
  assert_eq!(mesh.faces[1].source, 1);
  assert_eq!(mesh.bounds.max, Vector3d::new(1.0, 0.0, 1.0));

  let mut without_flags: BuildPrjFile = square(SOFT, SOFT)?;
  let mut past_vertices: BuildPrjFile = square(SOFT, SOFT)?;
  let mut of_a_glow: BuildPrjFile = square(SOFT, SOFT)?;

  without_flags.smoothing_groups = None;
  past_vertices.faces[0].vertices[2] = 9;
  of_a_glow.faces[1].material = 1;

  assert!(error_of(GeometryMesh::load(&without_flags)).contains("faces without smoothing flags"));
  assert!(
    error_of(GeometryMesh::load(&past_vertices)).contains("faces naming a vertex the project lacks: #0: vertex 9 of 4")
  );
  assert!(error_of(GeometryMesh::load(&of_a_glow)).contains("faces of a material with no compiler shader"));

  Ok(())
}

#[test]
fn welds_vertices_within_the_distance_strictly_and_drops_the_faces_it_closes() -> XrfResult {
  // Vertex 4 stands 0.4 mm from vertex 1, in the grid cell vertex 1 is filed under: within the 5 mm asked it welds,
  // and the face it closes against vertex 1 goes.
  let vertices: [[f32; 3]; 5] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [1.0004, 0.0, 0.0],
  ];
  let faces: [([u32; 3], u32); 3] = [([0, 2, 1], SOFT), ([0, 3, 2], SOFT), ([1, 2, 4], SOFT)];
  let mut mesh: GeometryMesh = GeometryMesh::load(&project(&vertices, &faces)?)?;
  let optimization: GeometryOptimization = mesh.optimize(0.005)?;

  assert_eq!(
    optimization,
    GeometryOptimization {
      welded_vertices: 1,
      degenerate_faces: 1,
      isolated_vertices: 1,
    }
  );
  assert_eq!((mesh.vertices.len(), mesh.faces.len()), (4, 2));

  // A distance of 0 welds nothing, the test being strict.
  let mut apart: GeometryMesh = GeometryMesh::load(&project(&vertices, &faces)?)?;

  assert_eq!(apart.optimize(0.0)?.welded_vertices, 0);
  assert_eq!(apart.faces.len(), 3);

  Ok(())
}

#[test]
fn welds_no_further_than_the_grid_reaches_across_a_cell_border() -> XrfResult {
  // Vertex 4 stands 4 mm past vertex 1, in the next of the grid's cells: vertex 1 reaches only 1 mm into it, so even a
  // 5 mm weld leaves them apart, as xrLC does.
  let vertices: [[f32; 3]; 5] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [1.004, 0.0, 0.0],
  ];
  let faces: [([u32; 3], u32); 3] = [([0, 2, 1], SOFT), ([0, 3, 2], SOFT), ([1, 2, 4], SOFT)];
  let mut mesh: GeometryMesh = GeometryMesh::load(&project(&vertices, &faces)?)?;

  assert_eq!(mesh.optimize(0.005)?.welded_vertices, 0);

  Ok(())
}

#[test]
fn refuses_a_face_of_no_area_and_one_mapped_to_a_point() -> XrfResult {
  // Three corners on one line.
  let mut flat: GeometryMesh = GeometryMesh::load(&project(
    &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
    &[([0, 1, 2], SOFT)],
  )?)?;

  assert!(error_of(flat.optimize(0.0)).contains("face 0 of no area"));

  let mut point: BuildPrjFile = square(SOFT, SOFT)?;

  point.faces[1].texture_coordinates = [Vector2d::new(0.5, 0.5); 3];

  assert!(error_of(GeometryMesh::load(&point)?.optimize(0.0)).contains("face 1 mapped to a point"));

  Ok(())
}

#[test]
fn cuts_a_long_edge_in_every_face_sharing_it_until_none_is_left() -> XrfResult {
  // A 10 m square over its 14.1 m diagonal: both faces are cut at the diagonal's middle, every edge then under 12.8 m.
  let mut mesh: GeometryMesh = GeometryMesh::load(&project(
    &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 0.0, 10.0], [0.0, 0.0, 10.0]],
    &[([0, 2, 1], SOFT), ([0, 3, 2], 0b001)],
  )?)?;

  mesh.optimize(0.0)?;

  assert_eq!(mesh.tessellate(), 2);
  assert_eq!((mesh.vertices.len(), mesh.faces.len()), (5, 4));
  assert_eq!(mesh.vertices[4], Vector3d::new(5.0, 0.0, 5.0));
  assert!(mesh.faces.iter().all(|face| {
    GeometryFace::EDGES.iter().all(|[from, to]| {
      mesh.vertices[face.vertices[*from] as usize].distance(&mesh.vertices[face.vertices[*to] as usize])
        < GeometryMesh::LONGEST_EDGE
    })
  }));
  // Each half keeps its face's material and source; the second face's hard edge `0 3` stays hard in its half, and the
  // new edge to the middle is soft.
  assert_eq!(
    mesh.faces.iter().map(|face| face.source).collect::<Vec<_>>(),
    [0, 0, 1, 1]
  );

  let holding = |first: u32, second: u32| {
    mesh
      .faces
      .iter()
      .find(|face| face.vertices.contains(&first) && face.vertices.contains(&second) && face.source == 1)
      .expect("a half holds the edge")
  };
  let edge_of = |face: &GeometryFace, first: u32, second: u32| {
    GeometryFace::EDGES
      .iter()
      .position(|[from, to]| {
        let ends: [u32; 2] = [face.vertices[*from], face.vertices[*to]];

        ends == [first, second] || ends == [second, first]
      })
      .expect("the face holds the edge")
  };
  let outer: &GeometryFace = holding(0, 3);
  let inner: &GeometryFace = holding(3, 4);

  assert!(!outer.is_soft_edge(edge_of(outer, 0, 3)));
  assert!(inner.is_soft_edge(edge_of(inner, 3, 4)));
  assert!(!inner.is_back_face());

  Ok(())
}

#[test]
fn shares_a_vertex_across_a_soft_edge_and_splits_it_across_a_hard_one_or_a_back_face() -> XrfResult {
  let normals = |first: u32, second: u32| -> XrfResult<GeometryNormals> {
    let mut mesh: GeometryMesh = GeometryMesh::load(&square(first, second)?)?;

    mesh.optimize(0.0)?;

    Ok(GeometryNormals::of(&mesh))
  };

  // The diagonal `0 2` is the first face's edge 0 and the second face's edge 2.
  let soft: GeometryNormals = normals(SOFT, SOFT)?;
  let hard: GeometryNormals = normals(0b001, SOFT)?;
  let back: GeometryNormals = normals(SOFT, GeometryFace::BACK_FACE)?;

  assert_eq!(soft.positions.len(), 4);
  assert_eq!(hard.positions.len(), 6);
  assert_eq!(back.positions.len(), 6);
  // `(v1 - v0) × (v2 - v1)` points up for both faces of the floor.
  assert!(
    soft
      .normals
      .iter()
      .all(|normal| *normal == Vector3d::new(0.0, 1.0, 0.0))
  );

  Ok(())
}

#[test]
fn packs_a_point_into_a_vertex_strictly_within_a_millimetre_found_in_its_own_cell() {
  // Cells along x are 10/24 m, the first border at 0.41667 m.
  let bounds: Aabb = Aabb {
    min: Vector3d::new(0.0, 0.0, 0.0),
    max: Vector3d::new(10.0, 10.0, 10.0),
  };
  let mut collector: GeometryCollector = GeometryCollector::new(&bounds);

  assert_eq!(collector.pack(&Vector3d::new(5.0, 5.0, 5.0)), 0);
  assert_eq!(collector.pack(&Vector3d::new(5.0005, 5.0, 5.0)), 0);
  assert_eq!(collector.pack(&Vector3d::new(5.002, 5.0, 5.0)), 1);

  // A vertex just below a border is filed in the cell above too, so a point just past the border finds it; one just
  // past it is filed only there, and a point just below does not.
  assert_eq!(collector.pack(&Vector3d::new(0.4163, 5.0, 5.0)), 2);
  assert_eq!(collector.pack(&Vector3d::new(0.4168, 5.0, 5.0)), 2);
  assert_eq!(collector.pack(&Vector3d::new(0.8338, 5.0, 5.0)), 3);
  assert_eq!(collector.pack(&Vector3d::new(0.8333, 5.0, 5.0)), 4);

  // A point below the box lands in the last cell, so it misses a vertex just inside the box's first.
  assert_eq!(collector.pack(&Vector3d::new(0.0004, 5.0, 5.0)), 5);
  assert_eq!(collector.pack(&Vector3d::new(-0.0001, 5.0, 5.0)), 6);
  assert_eq!(collector.into_vertices().len(), 7);
}

#[test]
fn collides_static_faces_of_a_colliding_shader_then_each_reference_placed_in_its_sector() -> XrfResult {
  // The square on the floor collides; a wall face of the `ghost` shader is only drawn.
  let mut file: BuildPrjFile = project(
    &[
      [0.0, 0.0, 0.0],
      [1.0, 0.0, 0.0],
      [1.0, 0.0, 1.0],
      [0.0, 0.0, 1.0],
      [0.0, 1.0, 0.0],
    ],
    &[([0, 2, 1], SOFT), ([0, 3, 2], SOFT), ([0, 1, 4], SOFT)],
  )?;
  let model_face = |material: u16, game_material: u32| BuildPrjFace {
    vertices: [0, 1, 2],
    texture_coordinates: [
      Vector2d::new(0.0, 0.0),
      Vector2d::new(1.0, 0.0),
      Vector2d::new(0.0, 1.0),
    ],
    material,
    material_padding: 0,
    game_material,
  };
  // A quarter turn about y, `x` to `-z` and `z` to `x`, then 2 m along x.
  let mut turn: Matrix4x4 = Matrix4x4::IDENTITY;

  turn.values[0] = 0.0;
  turn.values[2] = -1.0;
  turn.values[8] = 1.0;
  turn.values[10] = 0.0;
  turn.values[12] = 2.0;

  file.faces[2].material = 2;
  file.faces[1].game_material = 3;
  file.materials[0].sector = 4;
  file.mu_models = vec![BuildPrjMuModel {
    name: String::from("crate"),
    vertices: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(1.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
    ],
    // The game material keeps its low 14 bits; the ghost face is dropped.
    faces: vec![model_face(0, 0x4005), model_face(2, 0)],
    lod: u16::MAX,
    smoothing_groups: Some(vec![SOFT, SOFT]),
  }];
  file.mu_references = vec![BuildPrjMuReference {
    model: 0,
    transform: turn,
    flags: 0,
    sector: 7,
    sector_padding: 0,
    reserved: [0; 8],
  }];

  let shaders: CompilerShaders = CompilerShaders::resolve(&file, &library())?;
  let form: LevelCformGeometry = GeometryCollision::build(&GeometryMesh::load(&file)?, &file, &shaders)?;
  let face = |vertices: [u32; 3], material: u16, sector: u16| LevelCformFace {
    vertices,
    material,
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    sector,
  };

  assert_eq!(
    form.get_faces(),
    [face([0, 1, 2], 0, 4), face([0, 3, 1], 3, 4), face([4, 5, 6], 5, 7)]
  );
  assert_eq!(
    form.get_vertices()[4..],
    [
      Vector3d::new(2.0, 0.0, 0.0),
      Vector3d::new(2.0, 0.0, -1.0),
      Vector3d::new(3.0, 0.0, 0.0)
    ]
  );

  let mut astray: BuildPrjFile = file.clone();
  let mut ghostly: BuildPrjFile = file.clone();

  astray.mu_references[0].model = 9;
  ghostly.faces.iter_mut().for_each(|face| face.material = 2);

  assert!(
    error_of(GeometryCollision::build(
      &GeometryMesh::load(&astray)?,
      &astray,
      &shaders
    ))
    .contains("multi-use reference 0 names model 9 of 1")
  );
  assert!(
    error_of(GeometryCollision::build(
      &GeometryMesh::load(&ghostly)?,
      &ghostly,
      &shaders
    ))
    .contains("no static face to collide with")
  );

  Ok(())
}

#[test]
fn averages_the_normals_of_faces_joined_across_a_fold() -> XrfResult {
  // Two faces folded at a right angle along `1 2`, both facing out of the fold.
  let mut mesh: GeometryMesh = GeometryMesh::load(&project(
    &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 0.0]],
    &[([0, 1, 2], SOFT), ([1, 3, 2], SOFT)],
  )?)?;

  mesh.optimize(0.0)?;

  let normals: GeometryNormals = GeometryNormals::of(&mesh);
  let shared: Vector3d = normals.normals[normals.faces[0][1] as usize];

  assert_eq!(normals.positions.len(), 4);
  assert!((shared.length() - 1.0).abs() < 1e-6);
  assert!((shared.x.abs() - shared.y.abs()).abs() < 1e-6 && shared.z == 0.0);

  Ok(())
}
