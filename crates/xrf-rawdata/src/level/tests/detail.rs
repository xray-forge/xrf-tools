use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_editable_level::EditableDetailObjectTool;
use xrf_error::XrfResult;
use xrf_level::{DetailModel, DetailVertex, LevelDetailsFile, LevelDetailsHeader, LevelDetailsSlot};
use xrf_math::{Vector2d, Vector3d};
use xrf_test_utils::GeneratedResources;

use crate::level::build::level_build_atlas::{LevelBuildAtlas, LevelBuildAtlasPlace};
use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::build::level_build_library::LevelBuildLibrary;
use crate::level::build::level_build_textures::LevelBuildTextures;
use crate::level::level_detail_conversion::{LevelDetailConversion, LevelDetailDecompile};
use crate::level::level_detail_library::LevelDetailLibrary;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A model of one quad, `size` across, its coordinates `uv` gives each corner's.
fn quad(size: f32, uv: impl Fn(f32, f32) -> (f32, f32)) -> DetailModel {
  DetailModel {
    shader: String::from("details\\blend"),
    texture: String::from("detail\\detail_grass"),
    flags: 0,
    min_scale: 0.5,
    max_scale: 2.0,
    vertices: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
      .into_iter()
      .map(|(x, y)| {
        let (u, v) = uv(x, y);

        DetailVertex {
          position: Vector3d::new(x * size, y * size, 0.0),
          u,
          v,
        }
      })
      .collect(),
    indices: vec![0, 1, 2, 0, 2, 3],
  }
}

/// A slot planting `corners`' models, its base and height set and its light full.
fn slot(corners: [u8; 4]) -> [u32; 4] {
  let objects: u64 = corners
    .iter()
    .enumerate()
    .fold(0, |word, (corner, id)| word | (u64::from(*id) << (20 + 6 * corner)));
  let word: u64 = 1000 | (12 << 12) | objects | (0xFFFFF << 44);

  [word as u32, (word >> 32) as u32, 0x1234, 0x5678]
}

fn details(objects: Vec<DetailModel>, slots: Vec<[u32; 4]>) -> LevelDetailsFile {
  LevelDetailsFile {
    header: LevelDetailsHeader {
      version: LevelDetailsFile::CURRENT_VERSION,
      object_count: objects.len() as u32,
      offset_x: 4,
      offset_z: 2,
      size_x: slots.len() as u32,
      size_z: 1,
    },
    objects,
    slots,
  }
}

fn library() -> LevelDetailLibrary {
  LevelDetailLibrary::new(vec![
    (String::from("detail\\small"), quad(1.0, |u, v| (u, v))),
    (String::from("detail\\grass"), quad(2.0, |u, v| (u, v))),
  ])
}

/// A model the library draws comes back as its reference at the level's scale range and flags, its coordinates the
/// library's moved into the atlas, turned or not; a model it lacks is refused, and the corners planting it plant
/// nothing. The slots keep their heights and densities, their light zeroed.
#[test]
fn decompiles_details_into_the_library_objects_drawing_them() {
  let mut atlas: DetailModel = quad(2.0, |u, v| (0.25 * u + 0.5, 0.5 * v));
  let turned: DetailModel = quad(1.0, |u, v| (0.125 * v, 0.25 * u + 0.75));
  let unknown: DetailModel = quad(3.0, |u, v| (u, v));

  atlas.flags = 1;
  atlas.min_scale = 0.8;

  let file: LevelDetailsFile = details(
    vec![atlas, unknown, turned],
    vec![slot([0, 1, 2, 63]), slot([63, 63, 63, 63])],
  );
  let LevelDetailDecompile { tool, refused } = LevelDetailConversion::decompile(&file, &library());
  let tool: EditableDetailObjectTool = tool.expect("two models found");

  assert_eq!(
    tool
      .objects
      .iter()
      .map(|object| (object.reference.as_str(), object.flags, object.min_scale))
      .collect::<Vec<_>>(),
    [("detail\\grass", 1, 0.8), ("detail\\small", 0, 0.5)]
  );
  assert_eq!(refused.len(), 1);
  assert_eq!(refused[0].object, "detail object 1");
  assert_eq!(
    (0..LevelDetailsSlot::CORNERS)
      .map(|corner| LevelDetailsSlot::get_object(&tool.slots[0], corner))
      .collect::<Vec<_>>(),
    [Some(0), None, Some(1), None]
  );
  assert_eq!(tool.slots[0][1] >> 12, 0, "the light zeroed");
  assert_eq!((tool.slots[0][0] & 0xF_FFFF), 1000 | (12 << 12), "the height kept");
  assert_eq!(
    [tool.slots[0][2], tool.slots[0][3]],
    [0x1234, 0x5678],
    "the densities kept"
  );
  assert_eq!(
    (tool.header.object_count, tool.header.offset_x, tool.header.size_x),
    (2, 4, 2)
  );
}

/// Where the library draws none of the level's models, there is no tool: the Build would refuse one with nothing to
/// merge into its atlas.
#[test]
fn leaves_the_tool_out_where_no_model_is_found() {
  let file: LevelDetailsFile = details(vec![quad(3.0, |u, v| (u, v))], vec![slot([0, 63, 63, 63])]);
  let decompile: LevelDetailDecompile = LevelDetailConversion::decompile(&file, &LevelDetailLibrary::default());

  assert!(decompile.tool.is_none());
  assert_eq!(decompile.refused.len(), 1);
}

/// A model of the library's geometry whose coordinates no single remap reaches is another object's.
#[test]
fn matches_no_model_whose_coordinates_are_no_remap() {
  let bent: DetailModel = quad(2.0, |u, v| (u + 0.3 * v, v));

  assert_eq!(library().find(&bent), None);
  assert_eq!(library().find(&quad(2.0, |u, v| (u, v))), Some("detail\\grass"));
}

/// Textures go in largest first at the first free spot, row by row; one fitting only turned goes in turned, and the
/// atlas doubles its narrower side until all fit.
#[test]
fn merges_textures_as_the_editor_packs_its_detail_atlas() {
  let image = |width: u32, height: u32| RgbaImage::from_pixel(width, height, Rgba([200, 100, 50, 255]));
  let at = |x: f32, y: f32| Vector2d::new(x, y);
  let merge = |textures: &[RgbaImage]| {
    LevelBuildAtlas::merge(
      &textures.iter().map(std::slice::from_ref).collect::<Vec<_>>(),
      &LevelBuildAtlas::DETAIL_SIZES,
    )
  };
  let atlas: LevelBuildAtlas = merge(&[image(64, 32), image(256, 256), image(128, 512)]).expect("a fit");

  assert_eq!(atlas.layers[0].dimensions(), (512, 512));
  assert_eq!(
    atlas.places,
    [
      LevelBuildAtlasPlace {
        offset: at(0.75, 0.0),
        scale: at(0.125, 0.0625),
        is_rotated: false,
      },
      LevelBuildAtlasPlace {
        offset: at(0.25, 0.0),
        scale: at(0.5, 0.5),
        is_rotated: false,
      },
      LevelBuildAtlasPlace {
        offset: at(0.0, 0.0),
        scale: at(0.25, 1.0),
        is_rotated: false,
      },
    ],
    "the tall texture first, at the corner, then the square beside it and the small one beside that"
  );
  assert_eq!(atlas.places[0].remap(1.0, 1.0), (0.875, 0.0625));
  assert_eq!(*atlas.layers[0].get_pixel(384, 0), Rgba([200, 100, 50, 255]));
  assert_eq!(
    *atlas.layers[0].get_pixel(511, 511),
    Rgba([0, 0, 0, 0]),
    "the rest left clear"
  );

  let stacked: LevelBuildAtlas = merge(&[image(512, 128), image(512, 384)]).expect("a fit");

  assert_eq!(stacked.layers[0].dimensions(), (512, 512));
  assert_eq!(stacked.places[0].offset, at(0.0, 0.75), "the smaller below the larger");
  assert!(merge(&[image(2048, 16)]).is_none(), "past the largest atlas");

  let turned: LevelBuildAtlas = merge(&[image(256, 512)]).expect("a fit");

  assert_eq!(
    turned.layers[0].dimensions(),
    (512, 256),
    "the atlas widens before it grows tall"
  );
  assert!(turned.places[0].is_rotated);
  assert_eq!(turned.places[0].remap(0.0, 1.0), (1.0, 0.0), "turned, u and v swap");
}

/// Impostor strips merge with their normals as a second layer at the same spots, in an atlas starting one strip in
/// size that grows tall first.
#[test]
fn merges_layered_strips_as_the_build_packs_its_impostors() {
  let strip = |value: u8| RgbaImage::from_pixel(512, 64, Rgba([value, value, value, 255]));
  let (first, second) = ([strip(10), strip(20)], [strip(30), strip(40)]);
  let atlas: LevelBuildAtlas = LevelBuildAtlas::merge(&[&first, &second], &LevelBuildAtlas::LOD_SIZES).expect("a fit");

  assert_eq!(atlas.layers.len(), 2);
  assert_eq!(
    atlas.layers[1].dimensions(),
    (512, 128),
    "the narrower side, the height, doubles"
  );
  assert_eq!(atlas.places[1].offset, Vector2d::new(0.0, 0.5));
  assert_eq!(*atlas.layers[0].get_pixel(0, 64), Rgba([30, 30, 30, 255]));
  assert_eq!(
    *atlas.layers[1].get_pixel(0, 64),
    Rgba([40, 40, 40, 255]),
    "each layer at its texture's spot"
  );
}

/// The export keeps the planted objects alone, compiled from their library objects with their coordinates moved into
/// an atlas of their textures named `build_details`, and the slots renumbered with their light zeroed.
#[test]
fn exports_the_planted_details_with_their_atlas() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "details_export"))?;
  let (objects, textures) = (root.join("objects"), root.join("textures"));

  std::fs::create_dir_all(objects.join("detail"))?;
  std::fs::create_dir_all(textures.join("detail"))?;

  for (name, size) in [("grass", 2.0), ("small", 1.0)] {
    std::fs::write(
      objects.join("detail").join(format!("{name}.object")),
      ObjectDecompiler::decompile_detail(&quad(size, |u, v| (u, v)))?
        .object
        .write_to_bytes::<XRayByteOrder>()?,
    )?;
  }

  let texture: RgbaImage = RgbaImage::from_pixel(64, 32, Rgba([10, 200, 10, 255]));

  std::fs::write(
    textures.join("detail").join("detail_grass.dds"),
    DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
      .encode(&DdsMipChain::build(&texture, DdsMipmaps::Disabled)?)?
      .write_to_bytes()?,
  )?;

  // The small quad is the tool's second object, and no slot plants it.
  let file: LevelDetailsFile = details(
    vec![quad(2.0, |u, v| (u, v)), quad(1.0, |u, v| (u, v))],
    vec![slot([0, 63, 0, 63]), slot([63; 4])],
  );
  let tool: EditableDetailObjectTool = LevelDetailConversion::decompile(&file, &library())
    .tool
    .expect("both found");
  let exported: LevelBuildDetails = LevelBuildDetails::export::<XRayByteOrder>(
    &tool,
    &mut LevelBuildLibrary::new(&[objects], &[]),
    &LevelBuildTextures::new(Some(&textures), &[]),
  )?;
  let [model] = exported.file.objects.as_slice() else {
    panic!("the planted object alone: {}", exported.file.objects.len());
  };
  let corners = |model: &DetailModel| -> Vec<(Vector3d, f32, f32)> {
    model
      .vertices
      .iter()
      .map(|vertex| (vertex.position, vertex.u, vertex.v))
      .collect()
  };

  assert_eq!(tool.objects.len(), 2);
  assert_eq!(model.texture, LevelBuildDetails::ATLAS_TEXTURE);
  assert_eq!(
    corners(model),
    corners(&quad(2.0, |u, v| (0.25 * u, 0.125 * v))),
    "the 64 by 32 texture at the corner of a 256 by 256 atlas"
  );
  assert_eq!(
    (
      exported.file.header.object_count,
      exported.atlas_size,
      exported.textures
    ),
    (1, (256, 256), 1)
  );
  assert_eq!(
    (0..LevelDetailsSlot::CORNERS)
      .map(|corner| LevelDetailsSlot::get_object(&exported.file.slots[0], corner))
      .collect::<Vec<_>>(),
    [Some(0), None, Some(0), None]
  );
  assert_eq!(exported.file.slots[0][1] >> 12, 0, "the light zeroed");
  assert_eq!(
    DdsFile::read_from_bytes(&exported.atlas)?.decode_rgba(0)?.dimensions(),
    (256, 256)
  );

  Ok(())
}
