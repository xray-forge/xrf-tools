use xrf_error::XrfResult;
use xrf_level::{BuildPrjFile, BuildPrjPortal};
use xrf_math::{EPS_L, Vector2d, Vector3d};

use crate::geometry::visual::geometry_visual::GeometryVisual;
use crate::geometry::visual::geometry_visual_light::GeometryVisualLight;
use crate::tests::geometry::geometry_mesh::project;
use crate::{
  GeometryBasis, GeometryBounds, GeometryHierarchy, GeometryModels, GeometryNodeKind, GeometryVisualVertex,
  GeometryVisuals, LightColor,
};

/// A visual of one triangle at `x`, a metre a side, in `sector`.
fn visual_at(x: f32, sector: u16) -> GeometryVisual {
  let mut visual: GeometryVisual = GeometryVisual::new(
    0,
    sector,
    vec![String::from("ground")],
    String::new(),
    GeometryVisualLight::Vertex,
  );
  let corner = |position: [f32; 3]| GeometryVisualVertex {
    position: Vector3d::from(position),
    normal: Vector3d::new(0.0, 1.0, 0.0),
    basis: GeometryBasis::NONE,
    texture_coordinate: Vector2d::new(0.0, 0.0),
    lightmap_coordinate: None,
    light: LightColor::default(),
  };

  visual.vertices = vec![
    corner([x, 0.0, 0.0]),
    corner([x + 1.0, 0.0, 0.0]),
    corner([x, 0.0, 1.0]),
  ];
  visual.faces = vec![[0, 1, 2]];
  visual
}

fn no_models() -> GeometryModels {
  GeometryModels {
    models: Vec::new(),
    trees: Vec::new(),
    lods: Vec::new(),
  }
}

fn portal(front: u16, back: u16) -> BuildPrjPortal {
  BuildPrjPortal {
    sector_front: front,
    sector_back: back,
    vertex_slots: [Vector3d::new(0.0, 0.0, 0.0); BuildPrjPortal::MAXIMUM_VERTICES],
    vertex_count: 3,
  }
}

#[test]
fn builds_a_models_visuals_by_material_once_and_places_them_by_each_reference() -> XrfResult {
  use xrf_level::{BuildPrjFace, BuildPrjMuModel, BuildPrjMuReference};
  use xrf_math::Matrix4x4;

  use crate::tests::geometry::geometry_mesh::library;
  use crate::{CompilerShaders, CompilerTexture};

  let face = |vertices: [u32; 3], material: u16| BuildPrjFace {
    vertices,
    texture_coordinates: [
      Vector2d::new(0.0, 0.0),
      Vector2d::new(1.0, 0.0),
      Vector2d::new(0.0, 1.0),
    ],
    material,
    material_padding: 0,
    game_material: 0,
  };
  let mut file: BuildPrjFile = project(&[[0.0; 3]], &[])?;

  // A two-metre-high model of a floor face of material 2 and a wall face of material 0, the later one first by
  // material.
  file.mu_models = vec![BuildPrjMuModel {
    name: String::from("bush"),
    vertices: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(1.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
      Vector3d::new(0.0, 2.0, 0.0),
    ],
    faces: vec![face([0, 2, 1], 2), face([0, 1, 3], 0)],
    lod: u16::MAX,
    smoothing_groups: Some(vec![0, 0]),
  }];
  file.mu_references = (0..2)
    .map(|index| BuildPrjMuReference {
      model: 0,
      transform: Matrix4x4::from_translation(&Vector3d::new(10.0 * f32::from(index), 0.0, 0.0)),
      flags: 0,
      sector: index,
      sector_padding: 0,
      reserved: [0; 8],
    })
    .collect();

  let textures: Vec<CompilerTexture> = vec![CompilerTexture {
    name: String::from("ground"),
    width: 256,
    height: 256,
    has_alpha: false,
    has_image: false,
    is_implicitly_lit: false,
  }];
  let models: GeometryModels =
    GeometryModels::build(&file, &CompilerShaders::resolve(&file, &library())?, &textures, None)?;

  assert_eq!(models.models.len(), 1);
  assert_eq!(
    models.models[0]
      .visuals
      .iter()
      .map(|visual| visual.material)
      .collect::<Vec<_>>(),
    vec![0, 2]
  );
  assert_eq!(models.models[0].wind_base, Vector3d::new(0.5, 0.0, 0.5));
  assert_eq!(models.models[0].wind_height, 2.0);
  assert_eq!(
    models
      .trees
      .iter()
      .map(|tree| (tree.reference, tree.visual, tree.sector))
      .collect::<Vec<_>>(),
    vec![(0, 0, 0), (0, 1, 0), (1, 0, 1), (1, 1, 1)]
  );
  assert!(models.lods.is_empty());

  Ok(())
}

/// A model drawing nothing has no tree for a LOD to hold; a LOD made for it anyway would stop every later reference's
/// LOD from reaching the hierarchy, and with it their trees.
#[test]
fn gives_no_lod_to_a_model_that_draws_nothing_and_keeps_the_lods_after_it() -> XrfResult {
  use xrf_level::{BuildPrjFace, BuildPrjLod, BuildPrjLodFace, BuildPrjMuModel, BuildPrjMuReference};
  use xrf_math::Matrix4x4;
  use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

  use crate::{CompilerShaders, CompilerTexture};

  let model = |name: &str, material: u16| BuildPrjMuModel {
    name: String::from(name),
    vertices: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(1.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
    ],
    faces: vec![BuildPrjFace {
      vertices: [0, 2, 1],
      texture_coordinates: [Vector2d::new(0.0, 0.0); 3],
      material,
      material_padding: 0,
      game_material: 0,
    }],
    lod: 0,
    smoothing_groups: Some(vec![0]),
  };
  let reference = |model: u32, x: f32| BuildPrjMuReference {
    model,
    transform: Matrix4x4::from_translation(&Vector3d::new(x, 0.0, 0.0)),
    flags: 0,
    sector: 0,
    sector_padding: 0,
    reserved: [0; 8],
  };
  let mut file: BuildPrjFile = project(&[[0.0; 3]], &[])?;

  // Material 2's compiler shader, `ghost`, is neither drawn nor collides in this library.
  file.mu_models = vec![model("hidden", 2), model("bush", 0)];
  file.mu_references = vec![reference(0, 0.0), reference(1, 10.0)];
  file.lods = vec![BuildPrjLod {
    faces: std::array::from_fn(|_| BuildPrjLodFace {
      vertices: [Vector3d::new(0.0, 0.0, 0.0); 4],
      texture_coordinates: [Vector2d::new(0.0, 0.0); 4],
    }),
    material: 0,
  }];

  let shader = |name: &str, flags: u32| ShaderCompilerShader {
    name: String::from(name),
    name_trailing: vec![0; ShaderCompilerShader::NAME_SIZE - name.len()],
    flags,
    vertex_translucency: 0.5,
    vertex_ambient: 0.0,
    lightmap_density: 1.0,
  };
  let library: ShaderCompilerFile = ShaderCompilerFile {
    shaders: vec![shader("default", ShaderCompilerShader::RENDERING), shader("ghost", 0)],
  };
  let textures: Vec<CompilerTexture> = vec![CompilerTexture {
    name: String::from("ground"),
    width: 256,
    height: 256,
    has_alpha: false,
    has_image: false,
    is_implicitly_lit: false,
  }];
  let models: GeometryModels =
    GeometryModels::build(&file, &CompilerShaders::resolve(&file, &library)?, &textures, None)?;

  assert_eq!(models.lods.iter().map(|lod| lod.reference).collect::<Vec<_>>(), vec![1]);

  let visuals: GeometryVisuals = GeometryVisuals {
    visuals: vec![visual_at(100.0, 0)],
  };
  let hierarchy: GeometryHierarchy = GeometryHierarchy::build(&visuals, &models, &file)?;

  assert!(
    hierarchy.nodes.iter().any(|node| node.kind == GeometryNodeKind::Lod(0)),
    "the bush's LOD is in the tree"
  );

  Ok(())
}

#[test]
fn bounds_a_geometry_in_its_grown_box_and_a_sphere_holding_every_point() {
  let points: [Vector3d; 4] = [
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(4.0, 0.0, 0.0),
    Vector3d::new(0.0, 2.0, 0.0),
    Vector3d::new(4.0, 2.0, 1.0),
  ];
  let bounds: GeometryBounds = GeometryBounds::of(&points);

  assert_eq!(bounds.bounds.min, Vector3d::new(-EPS_L, -EPS_L, -EPS_L));
  assert_eq!(bounds.bounds.max, Vector3d::new(4.0 + EPS_L, 2.0 + EPS_L, 1.0 + EPS_L));
  assert!(
    points
      .iter()
      .all(|point| bounds.center.distance(point) <= bounds.radius + EPS_L)
  );
  // No larger than the box's half diagonal.
  assert!(bounds.radius <= (4.0f32 * 4.0 + 2.0 * 2.0 + 1.0).sqrt() / 2.0 + EPS_L);
}

#[test]
fn joins_a_sectors_near_visuals_under_a_node_and_lists_each_portal_on_both_sides() -> XrfResult {
  // Sector 0 holds two triangles a metre apart, sector 1 one, far off; a portal joins them.
  let visuals: GeometryVisuals = GeometryVisuals {
    visuals: vec![visual_at(0.0, 0), visual_at(2.0, 0), visual_at(100.0, 1)],
  };
  let mut file: BuildPrjFile = project(&[[0.0; 3]], &[])?;

  file.portals = vec![portal(0, 1)];

  let hierarchy: GeometryHierarchy = GeometryHierarchy::build(&visuals, &no_models(), &file)?;

  assert_eq!(hierarchy.sectors.len(), 2);
  assert_eq!(
    hierarchy.nodes[hierarchy.sectors[0].root as usize].kind,
    GeometryNodeKind::Hierarchy(vec![0, 1])
  );
  assert_eq!(hierarchy.sectors[1].root, 2, "a sector of one visual is rooted at it");
  assert_eq!(hierarchy.sectors[0].portals, vec![0]);
  assert_eq!(hierarchy.sectors[1].portals, vec![0]);

  Ok(())
}

#[test]
fn refuses_a_sector_holding_no_visual_and_a_portal_into_one_past_the_last() -> XrfResult {
  let mut file: BuildPrjFile = project(&[[0.0; 3]], &[])?;
  let skipping: GeometryVisuals = GeometryVisuals {
    visuals: vec![visual_at(0.0, 0), visual_at(2.0, 2)],
  };

  assert!(GeometryHierarchy::build(&skipping, &no_models(), &file).is_err());

  file.portals = vec![portal(0, 5)];

  let one: GeometryVisuals = GeometryVisuals {
    visuals: vec![visual_at(0.0, 0)],
  };

  assert!(GeometryHierarchy::build(&one, &no_models(), &file).is_err());

  Ok(())
}
