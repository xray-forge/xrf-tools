use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_level::{
  BuildPrjFace, BuildPrjFile, BuildPrjLod, BuildPrjLodFace, BuildPrjMuModel, BuildPrjMuReference, LevelSector,
  LevelVisualRecord,
};
use xrf_math::{Aabb, Matrix4x4, Vector2d, Vector3d};
use xrf_ogf::OgfModelType;

use crate::geometry::level::geometry_level::GeometryLevel;
use crate::geometry::level::geometry_level_request::GeometryLevelRequest;
use crate::geometry::mesh::geometry_face::GeometryFace;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::input::compiler_texture::CompilerTexture;
use crate::light::bake::light_quality::LightQuality;
use crate::tests::geometry::geometry_mesh::{library, project};
use crate::tests::light::light_quality::lit;

// A floor square, and a model of a floor face and a wall face placed once beside it with its LOD.
fn level_project() -> XrfResult<BuildPrjFile> {
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
  let lighting: BuildPrjFile = lit(1)?;
  let mut file: BuildPrjFile = project(
    &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    &[([0, 2, 1], 0), ([0, 3, 2], 0)],
  )?;

  file.light_controls = lighting.light_controls;
  file.static_lights = lighting.static_lights;
  file.dynamic_lights = lighting.dynamic_lights;
  file.mu_models = vec![BuildPrjMuModel {
    name: String::from("bush"),
    vertices: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(1.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
      Vector3d::new(0.0, 2.0, 0.0),
    ],
    faces: vec![face([0, 2, 1], 0), face([0, 1, 3], 2)],
    lod: 0,
    smoothing_groups: Some(vec![0, 0]),
  }];
  file.mu_references = vec![BuildPrjMuReference {
    model: 0,
    transform: Matrix4x4::from_translation(&Vector3d::new(5.0, 0.0, 0.0)),
    flags: 0,
    sector: 0,
    sector_padding: 0,
    reserved: [0; 8],
  }];
  file.lods = vec![BuildPrjLod {
    faces: std::array::from_fn(|_| BuildPrjLodFace {
      vertices: [Vector3d::new(5.0, 0.0, 0.0); 4],
      texture_coordinates: [Vector2d::new(0.0, 0.0); 4],
    }),
    material: 0,
  }];

  Ok(file)
}

#[test]
fn assembles_a_level_of_visuals_trees_under_their_lod_and_one_rooted_sector() -> XrfResult {
  let file: BuildPrjFile = level_project()?;
  let textures: Vec<CompilerTexture> = vec![CompilerTexture {
    name: String::from("ground"),
    width: 256,
    height: 256,
    has_alpha: false,
    has_image: false,
    is_implicitly_lit: false,
  }];
  let roots: Vec<PathBuf> = Vec::new();
  let level: GeometryLevel = GeometryLevel::compile(&GeometryLevelRequest {
    project: &file,
    library: &library(),
    textures: &textures,
    roots: &roots,
    quality: LightQuality::Draft,
    light: None,
    job: &JobHandle::inert(),
  })?;
  let records: &[LevelVisualRecord] = level.bundle.visuals.as_deref().unwrap_or_default();
  let of_type = |model_type: OgfModelType| -> Vec<usize> {
    (0..records.len())
      .filter(|index| records[*index].header.model_type == model_type.to_raw())
      .collect()
  };

  assert_eq!(level.bundle.header.xrlc_version, GeometryLevel::VERSION);
  assert_eq!(
    (level.census.visuals, level.census.trees, level.census.sectors),
    (1, 2, 1)
  );
  assert_eq!(of_type(OgfModelType::Normal).len(), 1, "the floor");

  // The LOD stands over its model's two trees, which lie side by side in the visuals.
  let lods: Vec<usize> = of_type(OgfModelType::Lod);

  assert_eq!(lods.len(), 1);

  let lod: &LevelVisualRecord = &records[lods[0]];
  let children: &[u32] = lod.children.as_deref().unwrap_or_default();

  assert_eq!(children.len(), 2);
  assert!(
    children
      .iter()
      .all(|child| [OgfModelType::TreeSt.to_raw(), OgfModelType::TreePm.to_raw()]
        .contains(&records[*child as usize].header.model_type)),
    "{children:?} are trees"
  );
  assert!(lod.lod.is_some() && lod.geometry.is_none());

  // A visual's geometry counts whole faces; the shader table opens with the empty entry the glows and visuals follow.
  for record in records.iter().filter(|record| record.geometry.is_some()) {
    assert_eq!(
      record.geometry.as_ref().map(|geometry| geometry.index_count % 3),
      Some(0)
    );
  }

  let shaders: usize = level.bundle.shaders.as_ref().map_or(0, |table| table.entries.len());

  assert!(
    records
      .iter()
      .all(|record| usize::from(record.header.shader_id) < shaders)
  );

  // The one sector is rooted at a visual that exists.
  let sectors: &[LevelSector] = level.bundle.sectors.as_deref().unwrap_or_default();

  assert_eq!(sectors.len(), 1);
  assert!((sectors[0].root as usize) < records.len());

  Ok(())
}

fn squares(xs: &[f32]) -> GeometryMesh {
  let mut vertices: Vec<Vector3d> = Vec::new();
  let mut faces: Vec<GeometryFace> = Vec::new();

  for x in xs {
    let base: u32 = vertices.len() as u32;

    vertices.extend([
      Vector3d::new(*x, 0.0, 0.0),
      Vector3d::new(x + 1.0, 0.0, 0.0),
      Vector3d::new(x + 1.0, 0.0, 1.0),
      Vector3d::new(*x, 0.0, 1.0),
    ]);

    for corners in [[base, base + 2, base + 1], [base, base + 3, base + 2]] {
      faces.push(GeometryFace {
        vertices: corners,
        texture_coordinates: [Vector2d::new(0.0, 0.0); 3],
        material: 0,
        game_material: 0,
        smoothing: 0,
        source: 0,
      });
    }
  }

  GeometryMesh {
    bounds: Aabb::from_points(&vertices).expect("a mesh of corners"),
    vertices,
    faces,
  }
}

#[test]
fn joins_small_splits_of_one_material_and_page_putting_the_taken_faces_first() {
  let mesh: GeometryMesh = squares(&[0.0, 2.0, 4.0]);
  let merged = |pages: Option<&[Option<u32>]>| {
    // An empty split, which a chart cut can leave, goes rather than being joined by a kind it does not have.
    let mut splits: GeometrySplits = GeometrySplits {
      splits: vec![vec![0, 1], Vec::new(), vec![2, 3], vec![4, 5]],
    };

    splits.merge(&mesh, pages);
    splits.splits
  };

  // The first takes the nearer, then the farther, each before its own faces.
  assert_eq!(merged(None), vec![vec![4, 5, 2, 3, 0, 1]]);
  // On a page of its own the middle square stays apart.
  assert_eq!(
    merged(Some(&[Some(0), Some(0), Some(1), Some(1), Some(0), Some(0)])),
    vec![vec![4, 5, 0, 1], vec![2, 3]]
  );
}
