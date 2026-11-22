use xrf_geometry::BvhTracer;
use xrf_level::{BuildPrjFile, BuildPrjName, BuildPrjShader};
use xrf_math::glam::Vec3;
use xrf_math::{Aabb, Vector2d, Vector3d};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::geometry::mesh::geometry_face::GeometryFace;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::input::compiler_shaders::CompilerShaders;
use crate::input::compiler_texture::CompilerTexture;
use crate::light::implicit::implicit_lighter::ImplicitLighter;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::atlas::lightmap_atlas::LightmapAtlas;
use crate::light::lightmap::atlas::lightmap_image_pair::LightmapImagePair;
use crate::light::lightmap::atlas::lightmap_page::LightmapPage;
use crate::light::lightmap::atlas::lightmap_placement::LightmapPlacement;
use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::lightmap::chart::light_chart_mask::LightChartMask;
use crate::light::lightmap::chart::light_chart_scene::LightChartScene;
use crate::light::lightmap::chart::lightmap_orientation::LightmapOrientation;
use crate::light::lightmap::lit::light_chart_lighter::LightChartLighter;
use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;
use crate::light::lightmap::lit::lightmap_resampler::LightmapResampler;
use crate::light::lightmap::lit::lit_chart::LitChart;
use crate::light::point::light_color::LightColor;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::ray::ray_scene::RayScene;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::tests::geometry::geometry_mesh::project;

fn face(vertices: [u32; 3]) -> GeometryFace {
  GeometryFace {
    vertices,
    texture_coordinates: [Vector2d::new(0.0, 0.0); 3],
    material: 0,
    game_material: 0,
    smoothing: 0,
    source: 0,
  }
}

fn mesh(vertices: Vec<Vector3d>, faces: Vec<GeometryFace>) -> GeometryMesh {
  GeometryMesh {
    bounds: Aabb::from_points(&vertices).expect("a mesh of corners"),
    vertices,
    faces,
  }
}

// The wall rises from the floor's far edge facing it, folded 90 degrees at a `lean` of 0 and opening as its top leans
// out.
pub(crate) fn floor_and_wall(lean: f32) -> GeometryMesh {
  mesh(
    vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(2.0, 0.0, 0.0),
      Vector3d::new(2.0, 0.0, 2.0),
      Vector3d::new(0.0, 0.0, 2.0),
      Vector3d::new(2.0, 2.0, 2.0 + lean),
      Vector3d::new(0.0, 2.0, 2.0 + lean),
    ],
    vec![face([0, 2, 1]), face([0, 3, 2]), face([3, 4, 2]), face([3, 5, 4])],
  )
}

fn tables() -> (BuildPrjFile, CompilerShaders, Vec<CompilerTexture>) {
  let shader = ShaderCompilerShader {
    name: String::from("default"),
    name_trailing: vec![0; ShaderCompilerShader::NAME_SIZE - 7],
    flags: ShaderCompilerShader::RENDERING,
    vertex_translucency: 0.5,
    vertex_ambient: 0.0,
    lightmap_density: 1.0,
  };
  let mut project: BuildPrjFile = project(&[[0.0; 3]], &[]).expect("a project");

  project.compiler_shaders = vec![BuildPrjShader {
    name: BuildPrjName::new("default", BuildPrjName::LONG_SIZE).expect("a name"),
  }];
  project.materials[0].compiler_shader = 0;
  project.materials[2].compiler_shader = 0;

  let shaders: CompilerShaders =
    CompilerShaders::resolve(&project, &ShaderCompilerFile { shaders: vec![shader] }).expect("resolve the shaders");
  let textures: Vec<CompilerTexture> = vec![CompilerTexture {
    name: String::from("ground"),
    width: 256,
    height: 256,
    has_alpha: false,
    has_image: false,
    is_implicitly_lit: false,
  }];

  (project, shaders, textures)
}

fn charts(mesh: &GeometryMesh) -> Vec<LightChart> {
  let (project, shaders, textures) = tables();
  let normals: GeometryNormals = GeometryNormals::of(mesh);
  let scene: LightChartScene<'_> = LightChartScene::of(mesh, &normals, &project, &shaders, &textures, 10.0, 1);
  let split: Vec<u32> = (0..mesh.faces.len() as u32).collect();

  LightChart::cut(&split, &scene)
}

fn triangle_chart(width: u32, height: u32) -> LightChart {
  LightChart {
    faces: vec![0],
    normal: Vector3d::new(0.0, 1.0, 0.0),
    projected: vec![[
      Vector2d::new(0.0, 0.0),
      Vector2d::new(1.0, 0.0),
      Vector2d::new(0.0, 1.0),
    ]],
    width,
    height,
    material: 0,
  }
}

#[test]
fn cuts_a_fold_past_the_smoothing_angle_into_a_chart_a_side_and_sizes_each_by_density() {
  let charts: Vec<LightChart> = charts(&floor_and_wall(0.0));

  // A floor face starts the first chart, whichever single precision makes the larger; the wall stands 90 degrees off
  // its normal, past 75 and a degree.
  let sorted = |chart: &LightChart| {
    let mut faces: Vec<u32> = chart.faces.clone();

    faces.sort_unstable();
    faces
  };

  assert_eq!(charts.len(), 2);
  assert_eq!(sorted(&charts[0]), vec![0, 1]);
  assert_eq!(sorted(&charts[1]), vec![2, 3]);
  // 2 m at 10 texels a metre and a density of 1, from a half texel up.
  assert_eq!((charts[0].width, charts[0].height), (21, 21));
  assert!((charts[0].normal.y - 1.0).abs() < 1e-6);
  assert!((charts[1].normal.z + 1.0).abs() < 1e-6);
}

#[test]
fn keeps_a_bend_within_the_smoothing_angle_on_one_chart() {
  // The wall leans 1.9 m out over 2 m up: about 46 degrees from the floor's normal.
  let charts: Vec<LightChart> = charts(&floor_and_wall(1.9));

  assert_eq!(charts.len(), 1);
  assert_eq!(charts[0].faces.len(), 4);
}

pub(crate) fn sun(energy: f32) -> LightSource {
  LightSource {
    is_directional: true,
    is_point: false,
    diffuse: Vec3::ONE,
    position: Vec3::ZERO,
    direction: Vec3::NEG_Y,
    range: 0.0,
    range_squared: 0.0,
    attenuation: [1.0, 0.0, 0.0],
    falloff: 0.0,
    energy,
  }
}

fn light(mesh: &GeometryMesh, lights: &LightSources) -> LitChart {
  light_by(mesh, lights, |lighter, floor| lighter.light(floor))
}

pub(crate) fn light_by<R>(
  mesh: &GeometryMesh,
  lights: &LightSources,
  lighting: impl FnOnce(&LightChartLighter<'_>, &LightChart) -> R,
) -> R {
  let (project, shaders, textures) = tables();
  let normals: GeometryNormals = GeometryNormals::of(mesh);
  let scene: LightChartScene<'_> = LightChartScene::of(mesh, &normals, &project, &shaders, &textures, 10.0, 1);
  let corners: Vec<Vec3> = mesh.vertices.iter().map(Vec3::from).collect();
  let triangles: Vec<[u32; 3]> = mesh.faces.iter().map(|face| face.vertices).collect();
  let tracer: BvhTracer = BvhTracer::new(&corners, &triangles);
  let ray_faces: Vec<Option<u32>> = (0..mesh.faces.len() as u32).map(Some).collect();
  let split: Vec<u32> = (0..mesh.faces.len() as u32).collect();
  let floor: LightChart = LightChart::cut(&split, &scene).remove(0);

  let lighter: LightChartLighter<'_> = LightChartLighter {
    scene: &scene,
    lights,
    rays: RayScene {
      tracer: &tracer,
      shadows: &ShadowMaterials::new_opaque(mesh.faces.len()),
    },
    ray_faces: &ray_faces,
    rms_zero: 2,
    rms_shrink: 4,
  };

  lighting(&lighter, &floor)
}

#[test]
fn lights_an_evenly_lit_chart_as_one_colour_two_texels_a_side_halved() {
  // The sun lights the floor, its own faces let through; the wall stands beside it, casting no shadow on it.
  let lit: LitChart = light(
    &floor_and_wall(0.0),
    &LightSources {
      sun: vec![sun(0.8)],
      ..LightSources::default()
    },
  );

  assert_eq!((lit.width, lit.height), (0, 0));
  assert_eq!((lit.layer.width, lit.layer.height), (2, 2));
  assert!(lit.layer.marker.iter().all(|marker| *marker == LightmapLayer::LIT));
  assert_eq!(lit.layer.surface[0].sun, LightColor::get_fixed(0.4));
  assert_eq!(lit.layer.surface[0].hemi, 0.0);
}

#[test]
fn lights_an_implicit_map_over_its_faces_texture_coordinates_and_keeps_the_texture_s_colour() {
  let mesh: GeometryMesh = floor_and_wall(0.0);
  let (project, shaders, textures) = tables();
  let normals: GeometryNormals = GeometryNormals::of(&mesh);
  let scene: LightChartScene<'_> = LightChartScene::of(&mesh, &normals, &project, &shaders, &textures, 10.0, 1);
  let corners: Vec<Vec3> = mesh.vertices.iter().map(Vec3::from).collect();
  let triangles: Vec<[u32; 3]> = mesh.faces.iter().map(|face| face.vertices).collect();
  let tracer: BvhTracer = BvhTracer::new(&corners, &triangles);
  let ray_faces: Vec<Option<u32>> = (0..mesh.faces.len() as u32).map(Some).collect();
  let lights: LightSources = LightSources {
    sun: vec![sun(0.8)],
    ..LightSources::default()
  };
  let shadows: ShadowMaterials = ShadowMaterials::new_opaque(mesh.faces.len());
  let lighter: LightChartLighter<'_> = LightChartLighter {
    scene: &scene,
    lights: &lights,
    rays: RayScene {
      tracer: &tracer,
      shadows: &shadows,
    },
    ray_faces: &ray_faces,
    rms_zero: 2,
    rms_shrink: 4,
  };
  // The floor's corners over the whole texture, by their x and z; the wall's beyond it.
  let coordinates: Vec<[Vector2d; 3]> = mesh
    .faces
    .iter()
    .map(|face| {
      face
        .get_corners(&mesh.vertices)
        .map(|corner| Vector2d::new(corner.x / 2.0, corner.z / 2.0 + corner.y))
    })
    .collect();
  let map: ImplicitMap = ImplicitLighter {
    lighter: &lighter,
    coordinates: &coordinates,
    job: &xrf_job::JobHandle::inert(),
  }
  .light(String::from("ground"), vec![[10, 20, 30, 255]; 16], 4, 4, &[0, 1])
  .expect("the CPU lights a map");

  // Every texel the sun halved, the texture's colour kept and its alpha the sky's, none here.
  assert_eq!((map.width, map.height), (4, 4));
  assert!(
    map
      .lightmap
      .iter()
      .all(|texel| texel[3] == LightColor::get_byte(LightColor::get_fixed(0.4)))
  );
  assert!(map.base.iter().all(|texel| *texel == [10, 20, 30, 0]));

  // A cancel stops the map before its first band, not after the whole map.
  let cancelled: xrf_job::JobHandle = xrf_job::JobHandle::inert();

  cancelled.cancel();

  assert!(
    ImplicitLighter {
      lighter: &lighter,
      coordinates: &coordinates,
      job: &cancelled,
    }
    .light(String::from("ground"), vec![[10, 20, 30, 255]; 16], 4, 4, &[0, 1])
    .is_err()
  );
}

#[test]
fn fills_a_ring_of_unlit_texels_with_their_lit_neighbours_mean() {
  let mut layer: LightmapLayer = LightmapLayer::new(3, 3);
  let lit = |sun: f32| LightColor {
    sun,
    ..LightColor::default()
  };

  layer.surface[0] = lit(1.0);
  layer.marker[0] = LightmapLayer::LIT;
  layer.surface[2] = lit(0.5);
  layer.marker[2] = LightmapLayer::LIT;

  assert!(layer.apply_borders(254));
  // Between the two: their mean; beside one alone: its light; the far corner, beside neither, waits for a ring more.
  assert_eq!(layer.surface[1].sun, LightColor::get_fixed(0.75));
  assert_eq!(
    (layer.surface[3].sun, layer.marker[3]),
    (LightColor::get_fixed(1.0), 254)
  );
  assert_eq!(layer.marker[8], 0);

  layer.fill_borders(253);
  assert!(layer.marker.iter().all(|marker| *marker != 0));
}

#[test]
fn rescales_rows_then_columns_and_keeps_an_image_at_its_own_size() {
  let pixels: Vec<[u8; 4]> = (0..16).map(|value| [value * 10, 200 - value * 5, 7, 100]).collect();

  // The cubic weighs a pixel's own 1 and its neighbours' 0: the image back as it was, but for the fourth byte, which
  // rounds half a step up on each of the two passes.
  let same: Vec<[u8; 4]> = LightmapResampler::Cubic.resample(&pixels, 4, 4, 4, 4);

  assert_eq!(
    same,
    pixels
      .iter()
      .map(|[r, g, b, a]| [*r, *g, *b, a + 2])
      .collect::<Vec<_>>()
  );

  let flat: Vec<[u8; 4]> = vec![[100; 4]; 64];
  let shrunk: Vec<[u8; 4]> = LightmapResampler::Lanczos3.resample(&flat, 8, 8, 4, 4);

  assert_eq!(shrunk.len(), 16);
  assert!(
    shrunk.iter().flatten().all(|byte| byte.abs_diff(100) <= 3),
    "{shrunk:?}"
  );
}

#[test]
fn masks_a_lit_layer_s_texels_marked_at_least_two_rings_out() {
  let mut layer: LightmapLayer = LightmapLayer::new(6, 4);

  for (index, marker) in [(7, 255), (8, 254), (9, 253), (10, 252)] {
    layer.marker[index] = marker;
  }

  let mask: LightChartMask = LightChartMask::of(&layer);

  assert_eq!((mask.width, mask.height), (6, 4));
  assert_eq!(mask.rows, vec![None, Some((1, 3)), None, None]);
  assert_eq!(mask.columns[1], Some((1, 1)));
  assert_eq!(mask.columns[4], None);
}

#[test]
fn places_each_mask_at_the_first_free_place_by_row_then_column() {
  let mut page: LightmapPage = LightmapPage::default();
  let block: Vec<Option<(u32, u32)>> = vec![Some((0, 9)); 10];

  assert_eq!(page.find(&block, 10, 0), Some((0, 0)));
  page.take(&block, 0, 0);
  assert_eq!(page.find(&block, 10, 0), Some((10, 0)));
  assert_eq!(page.find(&block, 10, 1), Some((10, 1)), "from a later row on");

  // A mask's empty corner lets another reach into it.
  let wedge: Vec<Option<(u32, u32)>> = (0..10).map(|row| Some((0, row))).collect();
  let mut page: LightmapPage = LightmapPage::default();

  page.take(&wedge, 0, 0);
  assert_eq!(page.find(&[Some((0, 3))], 4, 0), Some((1, 0)));
}

#[test]
fn packs_charts_onto_as_many_pages_as_they_need_none_overlapping() {
  let (project, shaders, textures) = tables();
  let mesh: GeometryMesh = floor_and_wall(0.0);
  let normals: GeometryNormals = GeometryNormals::of(&mesh);
  let scene: LightChartScene<'_> = LightChartScene::of(&mesh, &normals, &project, &shaders, &textures, 10.0, 1);
  let side: u32 = LightChart::MAXIMUM_SIDE;
  // Each a 510 texel square lit whole, 512 with borders: four to a page.
  let lit: LitChart = LitChart {
    width: side,
    height: side,
    layer: LightmapLayer::filled(side + 2, side + 2, LightColor::default()),
  };
  let atlas: LightmapAtlas = LightmapAtlas::pack(&vec![triangle_chart(side, side); 9], &vec![lit; 9], &scene);

  assert_eq!(atlas.pages, 3);

  for (index, first) in atlas.placements.iter().enumerate() {
    assert!(first.x + first.width <= LightChart::PAGE_SIDE && first.y + first.height <= LightChart::PAGE_SIDE);

    for second in &atlas.placements[index + 1..] {
      let apart: bool = first.page != second.page
        || first.x + first.width <= second.x
        || second.x + second.width <= first.x
        || first.y + first.height <= second.y
        || second.y + second.height <= first.y;

      assert!(apart, "{first:?} overlaps {second:?}");
    }
  }
}

#[test]
fn composes_a_page_of_its_charts_texels_crosswise_where_rotated_and_borders_round_them() {
  let lit = |width: u32, height: u32, sun: f32| LitChart {
    width: width - 2,
    height: height - 2,
    layer: LightmapLayer::filled(
      width,
      height,
      LightColor {
        sun,
        ..LightColor::default()
      },
    ),
  };
  let atlas: LightmapAtlas = LightmapAtlas {
    pages: 1,
    placements: vec![
      LightmapPlacement {
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 3,
        orientation: LightmapOrientation::Upright,
      },
      // A chart 3 wide and 5 high, laid 5 wide and 3 high.
      LightmapPlacement {
        page: 0,
        x: 10,
        y: 0,
        width: 5,
        height: 3,
        orientation: LightmapOrientation::Crosswise,
      },
    ],
  };
  let LightmapImagePair { first, second } = atlas.compose(0, &[lit(4, 3, 1.0), lit(3, 5, 0.5)]);
  let at = |x: usize, y: usize| y * LightChart::PAGE_SIDE as usize + x;

  assert_eq!(first[at(3, 2)][3], 255, "the first chart's sun in alpha");
  assert_eq!(
    second[at(3, 2)],
    [255, 255, 255, 0],
    "its sun in every colour, no hemisphere"
  );
  assert_eq!(first[at(14, 2)][3], 127, "the rotated chart's far corner");
  assert_eq!(first[at(15, 2)][3], 127, "a border ring past it");
  assert_eq!(first[at(40, 40)][3], 0, "beyond sixteen rings, nothing");
}

#[test]
fn maps_a_placed_chart_into_its_rectangle_less_the_border_a_half_texel_in() {
  let chart: LightChart = charts(&floor_and_wall(0.0)).remove(0);
  let coordinates: Vec<[Vector2d; 3]> = chart.remap([11, 21], [21, 21], [1024, 1024], LightmapOrientation::Upright);
  let all: Vec<&Vector2d> = coordinates.iter().flatten().collect();
  let low_x: f32 = all.iter().map(|corner| corner.x).fold(f32::INFINITY, f32::min);
  let high_x: f32 = all.iter().map(|corner| corner.x).fold(f32::NEG_INFINITY, f32::max);

  assert!((low_x - 11.5 / 1024.0).abs() < 1e-6);
  assert!((high_x - 31.5 / 1024.0).abs() < 1e-6);
}

#[test]
fn joins_splits_only_on_the_same_lightmap_page() {
  let (_, shaders, _) = tables();
  let mesh: GeometryMesh = floor_and_wall(0.0);
  let cut: GeometrySplits = GeometrySplits::cut(&mesh, &shaders, None).splits;
  let apart = |pages: [Option<u32>; 4]| {
    let mut splits: GeometrySplits = GeometrySplits {
      splits: vec![vec![0, 1], vec![2, 3]],
    };

    splits.merge(&mesh, Some(&pages));
    splits.splits.len()
  };

  assert_eq!(cut.splits, vec![vec![0, 1, 2, 3]]);
  assert_eq!(apart([Some(0), Some(0), Some(0), Some(0)]), 1);
  assert_eq!(apart([Some(0), Some(0), Some(1), Some(1)]), 2);
}
