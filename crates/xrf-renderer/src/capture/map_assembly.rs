use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use xrf_error::{XrfError, XrfResult};

use crate::capture::map_tile::MapTile;
use crate::contract::render_capture::RenderCapture;

/// A level map put together from its tiles as each comes back, reduced by Lanczos as `CDemoRecord` resamples.
pub struct MapAssembly {
  map: RgbaImage,
}

impl MapAssembly {
  /// An empty map `width` by `height` pixels, transparent until its tiles are put in.
  pub fn new(width: u32, height: u32) -> Self {
    Self {
      map: RgbaImage::new(width, height),
    }
  }

  /// Puts in a tile from what was drawn of it.
  ///
  /// # Errors
  ///
  /// Returns an error where what was drawn is not the tile's size.
  pub fn place(&mut self, tile: &MapTile, drawn: RenderCapture) -> XrfResult {
    let (width, height): (u32, u32) = tile.get_drawn_size();

    if (drawn.width, drawn.height) != (width, height) {
      return Err(XrfError::new_invalid_error(format!(
        "A map tile came back {} by {} pixels, not the {width} by {height} it was drawn at",
        drawn.width, drawn.height
      )));
    }

    let drawn: RgbaImage = RgbaImage::from_raw(width, height, drawn.pixels)
      .ok_or_else(|| XrfError::new_invalid_error(format!("A map tile's pixels do not fill {width} by {height}")))?;
    let margin: u32 = tile.margin;
    let (across, down): (u32, u32) = (tile.placed.width + 2 * margin, tile.placed.height + 2 * margin);
    let reduced: RgbaImage = if tile.supersampling > 1 {
      imageops::resize(&drawn, across, down, FilterType::Lanczos3)
    } else {
      drawn
    };
    let kept: RgbaImage =
      imageops::crop_imm(&reduced, margin, margin, tile.placed.width, tile.placed.height).to_image();

    imageops::replace(&mut self.map, &kept, i64::from(tile.placed.x), i64::from(tile.placed.y));

    Ok(())
  }

  /// The map, with every tile put in.
  pub fn finish(self) -> RgbaImage {
    self.map
  }
}
