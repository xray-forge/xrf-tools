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

/// Textures merged into one, as `CImageManager::CreateMergedTexture` (`ImageManagerDO.cpp:238`) merges the detail
/// objects' textures for `build_details.dds`: each texture placed largest first at the first free spot of a scan, row
/// by row, turned where only that fits, in an atlas starting at the smallest size and doubling its narrower side up to
/// the largest.
pub(crate) struct LevelBuildAtlas {
  pub(crate) image: RgbaImage,
  /// Each texture's place, in the order the textures were given.
  pub(crate) places: Vec<LevelBuildAtlasPlace>,
}

impl LevelBuildAtlas {
  /// The atlas sizes `EDetailManager::Export` allows, 256 to 1024 a side.
  pub(crate) const SMALLEST: u32 = 256;
  pub(crate) const LARGEST: u32 = 1024;

  /// `textures` merged, or `None` where they do not fit the largest atlas. The editor sorts by a predicate that is no
  /// ordering (larger area, or else a longer edge, either way round), so its order is its sort's own; this one places
  /// the larger area first, then the longer edge, then the given order.
  pub(crate) fn merge(textures: &[&RgbaImage]) -> Option<Self> {
    let (mut width, mut height) = (Self::SMALLEST, Self::SMALLEST);

    loop {
      if let Some(atlas) = Self::merge_into(textures, width, height) {
        return Some(atlas);
      }

      if width <= height && width < Self::LARGEST {
        width *= 2;
      } else if height < width && height < Self::LARGEST {
        height *= 2;
      } else {
        return None;
      }
    }
  }

  /// `textures` merged into an atlas `width` by `height`, or `None` where they do not fit.
  fn merge_into(textures: &[&RgbaImage], width: u32, height: u32) -> Option<Self> {
    let area: u64 = textures
      .iter()
      .map(|texture| u64::from(texture.width()) * u64::from(texture.height()))
      .sum();

    if area > u64::from(width) * u64::from(height) {
      return None;
    }

    let mut order: Vec<usize> = (0..textures.len()).collect();

    order.sort_by_key(|&index| {
      let texture: &RgbaImage = textures[index];

      (
        std::cmp::Reverse(u64::from(texture.width()) * u64::from(texture.height())),
        std::cmp::Reverse(texture.width().max(texture.height())),
      )
    });

    let mut mask: Vec<bool> = vec![false; (width * height) as usize];
    let mut image: RgbaImage = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    let mut places: Vec<Option<LevelBuildAtlasPlace>> = vec![None; textures.len()];

    for index in order {
      let texture: &RgbaImage = textures[index];
      let (x, y, is_rotated) = Self::place(&mut mask, (width, height), (texture.width(), texture.height()))?;
      let (placed_width, placed_height) = if is_rotated {
        (texture.height(), texture.width())
      } else {
        (texture.width(), texture.height())
      };

      for (source_x, source_y, pixel) in texture.enumerate_pixels() {
        let (to_x, to_y) = if is_rotated {
          (x + source_y, y + source_x)
        } else {
          (x + source_x, y + source_y)
        };

        image.put_pixel(to_x, to_y, *pixel);
      }

      places[index] = Some(LevelBuildAtlasPlace {
        offset: Vector2d::new(x as f32 / width as f32, y as f32 / height as f32),
        scale: Vector2d::new(placed_width as f32 / width as f32, placed_height as f32 / height as f32),
        is_rotated,
      });
    }

    Some(Self {
      image,
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
