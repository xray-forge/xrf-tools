use std::path::PathBuf;

use xrf_dds::{Rgba, RgbaImage};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::BuildPrjMaterial;
use xrf_math::{Aabb, Matrix4x4, Vector2d, Vector3d};
use xrf_ogf::{OgfLodDefinitionChunk, OgfLodFacet, OgfLodVertex};
use xrf_test_utils::GeneratedResources;

use crate::level::build::level_build_atlas::{LevelBuildAtlas, LevelBuildAtlasPlace};
use crate::level::build::level_build_impostors::LevelBuildImpostors;
use crate::level::build::level_build_models::LevelBuildModels;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;
use crate::level::level_lod_strip::LevelLodStrip;
use crate::level::level_tree_conversion::LevelTreeConversion;
use crate::level::level_tree_impostor::{LevelTreeImpostor, LevelTreeImpostorAtlas};
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A strip whose every pixel names its place and the strip it belongs to, so a cut that reads one pixel amiss shows.
fn strip(seed: u8) -> RgbaImage {
  RgbaImage::from_fn(LevelLodStrip::WIDTH, LevelLodStrip::HEIGHT, |x, y| {
    Rgba([(x % 256) as u8, (x / 256) as u8 * 64 + y as u8, seed, 255])
  })
}

/// The impostor a level draws from a strip at `place` in its atlas: each frame's corners at `bounds` placed by
/// `placement`, its coordinates moved to `place`.
fn lod(place: &LevelBuildAtlasPlace, bounds: &Aabb, placement: &Matrix4x4) -> OgfLodDefinitionChunk {
  OgfLodDefinitionChunk {
    facets: std::array::from_fn(|frame| {
      let (corners, coordinates) = (
        LevelLodStrip::get_frame_corners(frame, bounds),
        LevelLodStrip::get_frame_coordinates(frame),
      );

      OgfLodFacet {
        vertices: std::array::from_fn(|corner| {
          let (u, v) = place.remap(coordinates[corner].x, coordinates[corner].y);

          OgfLodVertex {
            position: placement.transform_point(&corners[corner]),
            texture_coordinate: Vector2d::new(u, v),
            rgb_hemi: 0,
            sun: 0,
          }
        }),
      }
    }),
  }
}

fn bounds(min: (f32, f32, f32), max: (f32, f32, f32)) -> Aabb {
  Aabb {
    min: Vector3d::new(min.0, min.1, min.2),
    max: Vector3d::new(max.0, max.1, max.2),
  }
}

/// A frame's quad stands through the box's middle, as wide as its widest half side and as tall as its half height, a
/// pixel in from each edge, turned an eighth of a circle per frame; the first frame's corners give the box back.
#[test]
fn stands_each_frame_through_the_box_and_reads_the_box_back() {
  let box_of: Aabb = bounds((-1.0, 0.0, -3.0), (3.0, 8.0, 1.0));
  let first: [Vector3d; 4] = LevelLodStrip::get_frame_corners(0, &box_of);
  let (offset_x, offset_y) = (1.0 - 1.0 / 512.0, 1.0 - 1.0 / 64.0);

  assert_eq!(
    first[0],
    Vector3d::new(1.0 - 2.0 * offset_x, 4.0 + 4.0 * offset_y, -1.0)
  );
  assert_eq!(
    first[2],
    Vector3d::new(1.0 + 2.0 * offset_x, 4.0 - 4.0 * offset_y, -1.0)
  );

  let quarter: [Vector3d; 4] = LevelLodStrip::get_frame_corners(2, &box_of);

  assert!(
    (quarter[0].x - 1.0).abs() < 1e-5 && (quarter[0].z - (-1.0 - 2.0 * offset_x)).abs() < 1e-5,
    "the third frame, a quarter turned, stands along z: {:?}",
    quarter[0]
  );

  let (center, across, half_height) = LevelLodStrip::get_first_frame_extent(&first);

  assert_eq!(center, Vector3d::new(1.0, 4.0, -1.0));
  assert!((across - 2.0).abs() < 1e-6 && (half_height - 4.0).abs() < 1e-6);
  assert_eq!(
    LevelLodStrip::get_frame_coordinates(1),
    [
      Vector2d::new(0.125 + 0.5 / 512.0, 0.5 / 64.0),
      Vector2d::new(0.25 - 0.5 / 512.0, 0.5 / 64.0),
      Vector2d::new(0.25 - 0.5 / 512.0, 1.0 - 0.5 / 64.0),
      Vector2d::new(0.125 + 0.5 / 512.0, 1.0 - 0.5 / 64.0),
    ],
    "the second cell, half a pixel in, corners clockwise from the top left"
  );
}

/// Each strip comes back out of the atlas the Build merged, from where its facets' coordinates name it, turned back
/// where the atlas holds it turned; facets naming no whole strip are refused.
#[test]
fn cuts_each_strip_back_out_of_the_atlas_where_its_facets_name_it() -> XrfResult {
  let (first, second) = ([strip(1), strip(101)], [strip(2), strip(102)]);
  let merged: LevelBuildAtlas = LevelBuildAtlas::merge(&[&first, &second], &LevelBuildAtlas::LOD_SIZES).expect("a fit");
  let atlas: LevelTreeImpostorAtlas = LevelTreeImpostorAtlas::new(merged.layers[0].clone(), merged.layers[1].clone())?;
  let box_of: Aabb = bounds((-1.0, 0.0, -1.0), (1.0, 4.0, 1.0));

  for (index, strips) in [&first, &second].into_iter().enumerate() {
    let cut: LevelTreeImpostor = atlas.cut(&lod(&merged.places[index], &box_of, &Matrix4x4::IDENTITY))?;

    assert_eq!(
      (cut.color == strips[0], cut.normal == strips[1]),
      (true, true),
      "strip {index}"
    );
  }

  // An atlas holding the strip turned a quarter, its rows as the atlas's columns.
  let turned: RgbaImage = RgbaImage::from_fn(LevelLodStrip::HEIGHT, LevelLodStrip::WIDTH, |x, y| {
    *first[0].get_pixel(y, x)
  });
  let turned_atlas: LevelTreeImpostorAtlas = LevelTreeImpostorAtlas::new(turned.clone(), turned)?;
  let place: LevelBuildAtlasPlace = LevelBuildAtlasPlace {
    offset: Vector2d::new(0.0, 0.0),
    scale: Vector2d::new(1.0, 1.0),
    is_rotated: true,
  };

  assert!(turned_atlas.cut(&lod(&place, &box_of, &Matrix4x4::IDENTITY))?.color == first[0]);

  let mut skewed: OgfLodDefinitionChunk = lod(&merged.places[0], &box_of, &Matrix4x4::IDENTITY);

  skewed.facets[3].vertices[1].texture_coordinate.x += 0.01;

  assert!(atlas.cut(&skewed).is_err(), "a frame off its cell names no strip");

  Ok(())
}

/// The model's stored box widens to the box its impostor was stood in, which held geometry the level does not draw,
/// read back through the placement; a recovered box not holding the model leaves it as it was.
#[test]
fn widens_the_models_box_to_the_one_its_impostor_stood_in() -> XrfResult {
  let triangle: [[Vector3d; 3]; 1] = [[
    Vector3d::new(-1.0, 1.0, -0.5),
    Vector3d::new(1.0, 5.0, -0.5),
    Vector3d::new(1.0, 1.0, 0.5),
  ]];
  let model = || -> XrfResult<EditableObjectFile> {
    Ok(
      ObjectDecompiler::decompile_level_geometry(&LevelVisualGeometry::of_triangles(
        "trees\\pine",
        "default",
        &SurfaceDress::default(),
        &triangle,
      ))?
      .object,
    )
  };
  let placement: Matrix4x4 = Matrix4x4::from_translation(&Vector3d::new(100.0, 2.0, -50.0))
    .multiply(&Matrix4x4::from_scale(&Vector3d::new(1.5, 1.5, 1.5)));
  let place: LevelBuildAtlasPlace = LevelBuildAtlasPlace {
    offset: Vector2d::new(0.0, 0.0),
    scale: Vector2d::new(1.0, 1.0),
    is_rotated: false,
  };
  // A trunk reaching a metre under the ground, which the level does not draw.
  let source_box: Aabb = bounds((-1.0, 0.0, -0.5), (1.0, 5.0, 0.5));
  let mut object: EditableObjectFile = model()?;

  assert!(LevelTreeConversion::fit_impostor_box(
    &mut object,
    &lod(&place, &source_box, &placement),
    &placement
  ));

  let widened: Aabb = object.meshes.iter().flatten().next().expect("a mesh").bounds;

  assert!(
    widened.min.distance(&source_box.min) < 1e-4 && widened.max.distance(&source_box.max) < 1e-4,
    "{widened:?}"
  );

  let mut untouched: EditableObjectFile = model()?;
  let before: Aabb = untouched.meshes.iter().flatten().next().expect("a mesh").bounds;

  assert!(!LevelTreeConversion::fit_impostor_box(
    &mut untouched,
    &lod(&place, &bounds((-1.0, 2.0, -0.5), (1.0, 5.0, 0.5)), &placement),
    &placement
  ));
  assert_eq!(untouched.meshes.iter().flatten().next().expect("a mesh").bounds, before);

  Ok(())
}

/// A multi-use object of an impostor gets eight quads around its box from the strips the decompile writes, its
/// material first, and a model naming it; the strips merge into the atlas its coordinates move into. Without a
/// textures root it is refused.
#[test]
fn builds_an_impostor_from_the_strips_the_decompile_writes() -> XrfResult {
  let library: GameMtlFile = GameMtlFile::new(vec![GameMtlMaterial::new(3, "materials\\tree")], Vec::new());
  let reference: &str = "levels\\test\\test_tree_0";
  let triangle: [[Vector3d; 3]; 1] = [[
    Vector3d::new(-1.0, 0.0, 0.0),
    Vector3d::new(1.0, 4.0, 0.0),
    Vector3d::new(1.0, 0.0, 1.0),
  ]];
  let mut object: EditableObjectFile = ObjectDecompiler::decompile_level_geometry(&LevelVisualGeometry::of_triangles(
    "trees\\pine",
    "default",
    &SurfaceDress {
      compiler_shader: String::from("default"),
      game_material: String::from("materials\\tree"),
    },
    &triangle,
  ))?
  .object;

  object.flags = EditableObjectFile::MULTIPLE_USAGE_FLAG | EditableObjectFile::USING_LOD_FLAG;

  let textures: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "impostor_build"))?;

  for texture in (LevelTreeImpostor {
    color: strip(5),
    normal: strip(6),
  })
  .into_textures(reference)?
  {
    let path: PathBuf = texture.get_path(&textures);

    std::fs::create_dir_all(path.parent().expect("a folder"))?;
    std::fs::write(path, texture.bytes)?;
  }

  let placed = xrf_editable_level::EditableSceneObject {
    object: xrf_editable_level::EditableCustomObject::new(
      String::from("tree_0000"),
      xrf_editable_level::EditableObjectTransform::default(),
    ),
    reference: String::from(reference),
    flags: 0,
  };
  let sectors: LevelBuildSectors = LevelBuildSectors::new(&[], |_, _| None);
  let mut tables: LevelBuildTables = LevelBuildTables::default();
  let mut models: LevelBuildModels = LevelBuildModels::default();

  assert!(
    models
      .add(&placed, &object, &sectors, &mut tables, &library, None)
      .is_err(),
    "an impostor needs the textures root"
  );

  models.add(&placed, &object, &sectors, &mut tables, &library, Some(&textures))?;

  assert_eq!(models.models[0].lod, 0);

  let records = tables.into_records()?;
  let first: &BuildPrjMaterial = &records.materials[0];

  assert_eq!(
    (
      records.textures[usize::from(first.texture)].name.value.as_str(),
      records.render_shaders[usize::from(first.shader)].name.value.as_str(),
      first.compiler_shader
    ),
    (
      LevelBuildImpostors::TEXTURE,
      LevelBuildImpostors::SHADER,
      LevelBuildTables::NO_COMPILER_SHADER
    ),
    "the impostor's material goes in before the model's"
  );

  let (lods, atlas) = models.impostors.into_export()?.expect("an impostor");
  let color: RgbaImage = xrf_dds::DdsFile::read_from_vec(atlas.color)?.decode_rgba(0)?;

  assert_eq!(
    color.dimensions(),
    (LevelLodStrip::WIDTH, LevelLodStrip::HEIGHT),
    "one strip, the smallest atlas"
  );
  assert_eq!(
    lods[0].faces[3].vertices,
    LevelLodStrip::get_frame_corners(3, &object.meshes.iter().flatten().next().expect("a mesh").bounds)
  );
  assert_eq!(
    lods[0].faces[3].texture_coordinates,
    LevelLodStrip::get_frame_coordinates(3),
    "an atlas of one strip moves no coordinate"
  );

  Ok(())
}
