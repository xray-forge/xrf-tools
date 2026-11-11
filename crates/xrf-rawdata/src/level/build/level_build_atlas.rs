use xrf_dds::{Rgba, RgbaImage};
use xrf_math::Vector2d;

/// Where one texture stands in an atlas: its corner and size as fractions of the atlas, and whether it was turned a
/// quarter to fit, its rows becoming the atlas's columns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelBuildAtlasPlace {
  pub(crate) offset: Vector2d,
  pub(crate) scale: Vector2d,
  pub(crate) is_rotated: bool,
}

impl LevelBuildAtlasPlace {
  /// A texture coordinate of the texture moved to its place in the atlas, `MergedTextureRemapUV`
  /// (`ImageManagerDO.cpp:352`).
  pub(crate) fn remap(&self, u: f32, v: f32) -> (f32, f32) {
    let (u, v) = if self.is_rotated { (v, u) } else { (u, v) };

    (self.scale.x * u + self.offset.x, self.scale.y * v + self.offset.y)
  }
}

/// The sizes an atlas may take, width and height: it starts at the smallest and doubles its narrower side up to the
/// largest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LevelBuildAtlasSizes {
  pub(crate) smallest: (u32, u32),
  pub(crate) largest: (u32, u32),
}

/// Textures merged into one, as `CImageManager::CreateMergedTexture` (`ImageManagerDO.cpp:145`, `:238`) merges the
/// detail objects' textures for `build_details.dds` and the impostors' strips and their normals for `level_lods.dds`
/// and `level_lods_nm.dds`: each texture placed largest first at the first free spot of a scan, row by row, turned
/// where only that fits, every layer of it at the same spot of its own image.
pub(crate) struct LevelBuildAtlas {
  /// One merged image per layer the textures hold.
  pub(crate) layers: Vec<RgbaImage>,
  /// Each texture's place, in the order the textures were given.
  pub(crate) places: Vec<LevelBuildAtlasPlace>,
}

impl LevelBuildAtlas {
  /// The atlas sizes `EDetailManager::Export` allows, 256 to 1024 a side.
  pub(crate) const DETAIL_SIZES: LevelBuildAtlasSizes = LevelBuildAtlasSizes {
    smallest: (256, 256),
    largest: (1024, 1024),
  };

  /// The atlas sizes the Build allows its impostors (`BuilderRemote.cpp:1541`), one strip to 2048 a side.
  pub(crate) const LOD_SIZES: LevelBuildAtlasSizes = LevelBuildAtlasSizes {
    smallest: (512, 64),
    largest: (2048, 2048),
  };

  /// `textures` merged, each given as its layers of one size, every texture of as many; `None` where they do not fit
  /// the largest of `sizes`. The editor sorts by a predicate that is no ordering (larger area, or else a longer edge,
  /// either way round), so its order is its sort's own; this one places the larger area first, then the longer edge,
  /// then the given order.
  pub(crate) fn merge(textures: &[&[RgbaImage]], sizes: &LevelBuildAtlasSizes) -> Option<Self> {
    let (mut width, mut height) = sizes.smallest;

    loop {
      if let Some(atlas) = Self::merge_into(textures, width, height) {
        return Some(atlas);
      }

      if width <= height && width < sizes.largest.0 {
        width *= 2;
      } else if height < width && height < sizes.largest.1 {
        height *= 2;
      } else {
        return None;
      }
    }
  }

  /// `textures` merged into an atlas `width` by `height`, or `None` where they do not fit.
  fn merge_into(textures: &[&[RgbaImage]], width: u32, height: u32) -> Option<Self> {
    let size = |index: usize| textures[index].first().map_or((0, 0), RgbaImage::dimensions);
    let area: u64 = (0..textures.len())
      .map(|index| u64::from(size(index).0) * u64::from(size(index).1))
      .sum();

    if area > u64::from(width) * u64::from(height) {
      return None;
    }

    let mut order: Vec<usize> = (0..textures.len()).collect();

    order.sort_by_key(|&index| {
      let (texture_width, texture_height) = size(index);

      (
        std::cmp::Reverse(u64::from(texture_width) * u64::from(texture_height)),
        std::cmp::Reverse(texture_width.max(texture_height)),
      )
    });

    let mut mask: Vec<bool> = vec![false; (width * height) as usize];
    let layer_count: usize = textures.first().map_or(0, |layers| layers.len());
    let mut layers: Vec<RgbaImage> = vec![RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0])); layer_count];
    let mut places: Vec<Option<LevelBuildAtlasPlace>> = vec![None; textures.len()];

    for index in order {
      let (texture_width, texture_height) = size(index);
      let (x, y, is_rotated) = Self::place(&mut mask, (width, height), (texture_width, texture_height))?;
      let (placed_width, placed_height) = if is_rotated {
        (texture_height, texture_width)
      } else {
        (texture_width, texture_height)
      };

      for (layer, texture) in layers.iter_mut().zip(textures[index]) {
        for (source_x, source_y, pixel) in texture.enumerate_pixels() {
          let (to_x, to_y) = if is_rotated {
            (x + source_y, y + source_x)
          } else {
            (x + source_x, y + source_y)
          };

          layer.put_pixel(to_x, to_y, *pixel);
        }
      }

      places[index] = Some(LevelBuildAtlasPlace {
        offset: Vector2d::new(x as f32 / width as f32, y as f32 / height as f32),
        scale: Vector2d::new(placed_width as f32 / width as f32, placed_height as f32 / height as f32),
        is_rotated,
      });
    }

    Some(Self {
      layers,
      places: places.into_iter().collect::<Option<_>>()?,
    })
  }

  /// `_rect_place`: the first free corner, scanning rows top down, where a rectangle `size` fits, else where it fits
  /// turned; the rectangle marked taken.
  fn place(mask: &mut [bool], (width, height): (u32, u32), size: (u32, u32)) -> Option<(u32, u32, bool)> {
    for (is_rotated, (rect_width, rect_height)) in [(false, size), (true, (size.1, size.0))] {
      if rect_width > width || rect_height > height {
        continue;
      }

      for y in 0..=height - rect_height {
        for x in 0..=width - rect_width {
          if !mask[(y * width + x) as usize] && Self::is_free(mask, width, (x, y), (rect_width, rect_height)) {
            for row in y..y + rect_height {
              mask[(row * width + x) as usize..(row * width + x + rect_width) as usize].fill(true);
            }

            return Some((x, y, is_rotated));
          }
        }
      }
    }

    None
  }

  fn is_free(mask: &[bool], width: u32, (x, y): (u32, u32), (rect_width, rect_height): (u32, u32)) -> bool {
    (y..y + rect_height).all(|row| {
      !mask[(row * width + x) as usize..(row * width + x + rect_width) as usize]
        .iter()
        .any(|taken| *taken)
    })
  }
}
