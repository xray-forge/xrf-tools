use xrf_dds::{DdsFile, Rgba, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector2d;
use xrf_ogf::OgfLodDefinitionChunk;

use crate::level::build::level_build_atlas::LevelBuildAtlasPlace;
use crate::level::level_lod_strip::LevelLodStrip;
use crate::level::scene::level_scene_decompile::LevelSceneTextureFile;

/// A level's impostor atlas, `level_lods.dds` and `level_lods_nm.dds`: every multi-use object's strip of eight views
/// and of their normals, which the Build merged into one (`SceneBuilder::CompileStatic`, `BuilderRemote.cpp:1516`)
/// and whose places the `MT_LOD` visuals' facets name.
pub struct LevelTreeImpostorAtlas {
  pub(crate) color: RgbaImage,
  pub(crate) normal: RgbaImage,
}

/// One object's impostor strips, as the Build loads them (`lod\lod_<object>` and its `_nm`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LevelTreeImpostor {
  pub(crate) color: RgbaImage,
  pub(crate) normal: RgbaImage,
}

impl LevelTreeImpostor {
  /// The strips as the textures the Build loads for the model of `reference`, `lod\lod_<model>` and its `_nm`, each
  /// encoded as the atlas they were cut from ([`LevelLodStrip::encode`]).
  ///
  /// # Errors
  ///
  /// Returns an error when a strip does not encode.
  pub(crate) fn into_textures(self, reference: &str) -> XrfResult<[LevelSceneTextureFile; 2]> {
    let name: String = LevelLodStrip::get_texture_name(reference);

    Ok([
      LevelSceneTextureFile {
        bytes: LevelLodStrip::encode(&self.color)?,
        name: name.clone(),
      },
      LevelSceneTextureFile {
        bytes: LevelLodStrip::encode(&self.normal)?,
        name: name + LevelLodStrip::NORMAL_SUFFIX,
      },
    ])
  }
}

impl LevelTreeImpostorAtlas {
  /// The files the atlas is shipped as, in the level's directory.
  pub(crate) const COLOR_FILE_NAME: &'static str = "level_lods.dds";
  pub(crate) const NORMAL_FILE_NAME: &'static str = "level_lods_nm.dds";

  /// How far a facet's coordinate may stand from where the strip's place puts it, past rounding.
  const TOLERANCE: f32 = 1e-4;

  /// The atlas of the two DDS files, `level_lods.dds` and `level_lods_nm.dds`, each read at its full size.
  ///
  /// # Errors
  ///
  /// Returns an error when a file is no DDS that decodes, or the two differ in size.
  pub fn read_from_bytes(color: Vec<u8>, normal: Vec<u8>) -> XrfResult<Self> {
    let decode = |bytes: Vec<u8>| DdsFile::read_from_vec(bytes).and_then(|file| file.decode_rgba(0));

    Self::new(decode(color)?, decode(normal)?)
  }

  /// The atlas of the two images, which must be the same size.
  ///
  /// # Errors
  ///
  /// Returns an error when the images differ in size.
  pub(crate) fn new(color: RgbaImage, normal: RgbaImage) -> XrfResult<Self> {
    if color.dimensions() != normal.dimensions() {
      return Err(XrfError::new_invalid_error(format!(
        "The impostor atlas is {:?} and its normals {:?}",
        color.dimensions(),
        normal.dimensions()
      )));
    }

    Ok(Self { color, normal })
  }

  /// The strips of the impostor `lod` draws: where its strip stands in the atlas, found from its first facet's
  /// corners turned or not, checked against every facet's, and cut back out of both images.
  ///
  /// # Errors
  ///
  /// Returns an error when no place of a whole strip in the atlas puts every facet's corners where they are.
  pub(crate) fn cut(&self, lod: &OgfLodDefinitionChunk) -> XrfResult<LevelTreeImpostor> {
    let place: LevelBuildAtlasPlace = [false, true]
      .into_iter()
      .filter_map(|is_rotated| self.fit(lod, is_rotated))
      .next()
      .ok_or_else(|| {
        XrfError::new_invalid_error("The impostor's facets name no strip of eight frames placed whole in the atlas")
      })?;
    let (width, height) = self.color.dimensions();
    let (x, y) = (
      (place.offset.x * width as f32).round() as u32,
      (place.offset.y * height as f32).round() as u32,
    );
    let cut = |image: &RgbaImage| -> RgbaImage {
      RgbaImage::from_fn(LevelLodStrip::WIDTH, LevelLodStrip::HEIGHT, |strip_x, strip_y| {
        let (at_x, at_y) = if place.is_rotated {
          (x + strip_y, y + strip_x)
        } else {
          (x + strip_x, y + strip_y)
        };

        image
          .get_pixel_checked(at_x, at_y)
          .copied()
          .unwrap_or(Rgba([0, 0, 0, 0]))
      })
    };

    Ok(LevelTreeImpostor {
      color: cut(&self.color),
      normal: cut(&self.normal),
    })
  }

  /// The strip's place fitted from the first facet's top left and bottom right corners, where it puts every corner of
  /// every facet where it is and covers a whole strip of pixels.
  fn fit(&self, lod: &OgfLodDefinitionChunk, is_rotated: bool) -> Option<LevelBuildAtlasPlace> {
    let source: [Vector2d; 4] = LevelLodStrip::get_frame_coordinates(0);
    let stored = |frame: usize, corner: usize| lod.facets[frame].vertices[corner].texture_coordinate;
    let (from, to) = (source[0], source[2]);
    let (from, to) = if is_rotated {
      (Vector2d::new(from.y, from.x), Vector2d::new(to.y, to.x))
    } else {
      (from, to)
    };
    let (start, end) = (stored(0, 0), stored(0, 2));
    let scale: Vector2d = Vector2d::new((end.x - start.x) / (to.x - from.x), (end.y - start.y) / (to.y - from.y));
    let place: LevelBuildAtlasPlace = LevelBuildAtlasPlace {
      offset: Vector2d::new(start.x - scale.x * from.x, start.y - scale.y * from.y),
      scale,
      is_rotated,
    };
    let (width, height) = self.color.dimensions();
    let placed: (f32, f32) = (scale.x * width as f32, scale.y * height as f32);
    let expected: (f32, f32) = if is_rotated {
      (LevelLodStrip::HEIGHT as f32, LevelLodStrip::WIDTH as f32)
    } else {
      (LevelLodStrip::WIDTH as f32, LevelLodStrip::HEIGHT as f32)
    };

    if (placed.0 - expected.0).abs() > 0.5 || (placed.1 - expected.1).abs() > 0.5 {
      return None;
    }

    let fits: bool = (0..LevelLodStrip::FRAMES).all(|frame| {
      LevelLodStrip::get_frame_coordinates(frame)
        .iter()
        .enumerate()
        .all(|(corner, coordinate)| {
          let (u, v) = place.remap(coordinate.x, coordinate.y);
          let at: Vector2d = stored(frame, corner);

          (u - at.x).abs() <= Self::TOLERANCE && (v - at.y).abs() <= Self::TOLERANCE
        })
    });

    fits.then_some(place)
  }
}
