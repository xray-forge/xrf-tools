//! Lightmap charts: how faces are cut into charts and sized, the texels a chart's mask takes, and how pages are packed.

use xrf_level::{BuildPrjFile, BuildPrjName, BuildPrjShader};
use xrf_math::{Aabb, Vector2d, Vector3d};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::chart::lightmap_page::LightmapPage;
use crate::tests::geometry::project;
use crate::{
  CompilerShaders, CompilerTexture, GeometryFace, GeometryMesh, GeometryNormals, GeometrySplits, LightChart,
  LightChartMask, LightChartScene, LightmapAtlas,
};

/// A face of material 0 over `vertices`.
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

/// A floor 2 m square facing up, and a wall 2 m high rising from its far edge facing it, its top `lean` metres further
/// out: upright at 0, folded 90 degrees from the floor, and opening the fold the further it leans.
fn floor_and_wall(lean: f32) -> GeometryMesh {
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

/// The geometry tests' project, its material 0 rendered and lit at a light density of 1.
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

/// The charts `mesh`'s one split is cut into at 10 texels a metre.
fn charts(mesh: &GeometryMesh) -> Vec<LightChart> {
  let (project, shaders, textures) = tables();
  let normals: GeometryNormals = GeometryNormals::of(mesh);
  let scene: LightChartScene<'_> = LightChartScene::of(mesh, &normals, &project, &shaders, &textures, 10.0, 1);
  let split: Vec<u32> = (0..mesh.faces.len() as u32).collect();

  LightChart::cut(&split, &scene)
}

/// A chart of one triangle `width` by `height` texels, its corners across its rectangle's lower left half.
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

#[test]
fn masks_a_chart_s_lit_texels_and_two_rings_within_its_bordered_rectangle() {
  let square: LightChartMask = LightChartMask::of(&charts(&floor_and_wall(0.0))[0], 1);

  assert_eq!((square.width, square.height), (23, 23));
  assert!(square.rows.iter().all(|row| *row == Some((0, 22))));

  // A triangle takes its lower left half, the rings reaching two texels past its edge but not outside its rectangle.
  let triangle: LightChartMask = LightChartMask::of(&triangle_chart(20, 20), 1);

  assert_eq!(triangle.rows[0], Some((0, 21)));
  assert_eq!(triangle.rows[21].map(|(first, _)| first), Some(0));
  assert!(triangle.rows[21].is_some_and(|(_, last)| last < 5));

  // A chart a texel across is its whole strip.
  let strip: LightChartMask = LightChartMask::of(&triangle_chart(1, 6), 1);

  assert_eq!((strip.width, strip.height), (2, 8));
  assert!(strip.columns.iter().all(|column| *column == Some((0, 7))));
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
  // Each a 510 texel square, 512 with borders: four to a page.
  let square: LightChart = LightChart {
    projected: vec![
      [
        Vector2d::new(0.0, 0.0),
        Vector2d::new(1.0, 0.0),
        Vector2d::new(1.0, 1.0),
      ],
      [
        Vector2d::new(0.0, 0.0),
        Vector2d::new(1.0, 1.0),
        Vector2d::new(0.0, 1.0),
      ],
    ],
    faces: vec![0, 1],
    ..triangle_chart(LightChart::MAXIMUM_SIDE, LightChart::MAXIMUM_SIDE)
  };
  let atlas: LightmapAtlas = LightmapAtlas::pack(&vec![square; 9], &scene);

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
fn maps_a_placed_chart_into_its_rectangle_less_the_border_a_half_texel_in() {
  let chart: LightChart = charts(&floor_and_wall(0.0)).remove(0);
  let coordinates: Vec<[Vector2d; 3]> = chart.remap([11, 21], [21, 21], [1024, 1024], false);
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
  let (cut, _) = GeometrySplits::cut(&mesh, &shaders, None);
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
