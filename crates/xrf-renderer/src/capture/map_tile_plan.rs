use std::cmp::Ordering;

use xrf_error::{XrfError, XrfResult};

use crate::capture::map_tile::MapTile;
use crate::contract::render_map_capture::{
  RENDER_MAP_MOST_PIXELS, RENDER_MAP_MOST_SUPERSAMPLING, RENDER_MAP_TILE, RENDER_MAP_TILE_MARGIN, RenderMapCapture,
};
use crate::contract::render_rect::RenderRect;

/// How a level map is drawn: its tiles row by row from its top left, after a metering view of the whole.
#[derive(Clone, Debug, PartialEq)]
pub struct MapTilePlan {
  pub capture: RenderMapCapture,
  pub tiles: Vec<MapTile>,
}

impl MapTilePlan {
  /// The most pixels the metering view spans on its longer side.
  pub const METER: u32 = 512;

  /// Plans `capture` for a device whose textures are at most `most_texture` pixels a side.
  ///
  /// # Errors
  ///
  /// Returns an error for an area or heights spanning nothing, or a map, supersampling or tile past what is drawn.
  pub fn new(capture: RenderMapCapture, most_texture: u32) -> XrfResult<Self> {
    let RenderMapCapture {
      min_x,
      min_z,
      max_x,
      max_z,
      bottom,
      top,
      width,
      height,
      supersampling,
      is_grassed,
      ..
    } = capture;

    if min_x.partial_cmp(&max_x) != Some(Ordering::Less) || min_z.partial_cmp(&max_z) != Some(Ordering::Less) {
      return Err(XrfError::new_invalid_error(format!(
        "A map's area must span some ground, not {min_x}, {min_z} to {max_x}, {max_z}"
      )));
    }

    if bottom.partial_cmp(&top) != Some(Ordering::Less) {
      return Err(XrfError::new_invalid_error(format!(
        "A map's heights must span some height, not {bottom} to {top}"
      )));
    }

    if width == 0 || height == 0 || width > RENDER_MAP_MOST_PIXELS || height > RENDER_MAP_MOST_PIXELS {
      return Err(XrfError::new_invalid_error(format!(
        "A map must be 1 to {} pixels a side, not {width} by {height}",
        RENDER_MAP_MOST_PIXELS
      )));
    }

    if supersampling == 0 || supersampling > RENDER_MAP_MOST_SUPERSAMPLING {
      return Err(XrfError::new_invalid_error(format!(
        "A map is drawn 1 to {} times over, not {supersampling}",
        RENDER_MAP_MOST_SUPERSAMPLING
      )));
    }

    let drawn: u32 = (RENDER_MAP_TILE.min(width.max(height)) + 2 * RENDER_MAP_TILE_MARGIN) * supersampling;

    if drawn > most_texture {
      return Err(XrfError::new_invalid_error(format!(
        "A tile drawn {supersampling} times over takes {drawn} pixels, past the {most_texture} the GPU draws"
      )));
    }

    // One map pixel's ground, which may differ slightly between the axes: the map covers the area exactly.
    let (pixel_x, pixel_z): (f32, f32) = ((max_x - min_x) / width as f32, (max_z - min_z) / height as f32);
    let margin: f32 = RENDER_MAP_TILE_MARGIN as f32;
    let mut tiles: Vec<MapTile> = Vec::new();

    for y in (0..height).step_by(RENDER_MAP_TILE as usize) {
      for x in (0..width).step_by(RENDER_MAP_TILE as usize) {
        let placed: RenderRect = RenderRect {
          x: x as i32,
          y: y as i32,
          width: RENDER_MAP_TILE.min(width - x),
          height: RENDER_MAP_TILE.min(height - y),
        };

        // The map's rows run from its top, the greatest `z`, down.
        tiles.push(MapTile {
          placed,
          margin: RENDER_MAP_TILE_MARGIN,
          supersampling,
          min_x: min_x + (x as f32 - margin) * pixel_x,
          max_x: min_x + ((x + placed.width) as f32 + margin) * pixel_x,
          min_z: max_z - ((y + placed.height) as f32 + margin) * pixel_z,
          max_z: max_z - (y as f32 - margin) * pixel_z,
          bottom,
          top,
          is_grassed,
        });
      }
    }

    Ok(Self { capture, tiles })
  }

  /// The view its exposure is metered over first: the whole area, at most `METER` pixels, with no margin.
  pub fn get_meter(&self) -> MapTile {
    let RenderMapCapture {
      min_x,
      min_z,
      max_x,
      max_z,
      bottom,
      top,
      width,
      height,
      ..
    } = self.capture;
    let scale: f32 = (Self::METER as f32 / width.max(height) as f32).min(1.0);

    MapTile {
      placed: RenderRect {
        x: 0,
        y: 0,
        width: ((width as f32 * scale).round() as u32).max(1),
        height: ((height as f32 * scale).round() as u32).max(1),
      },
      margin: 0,
      supersampling: 1,
      min_x,
      max_x,
      min_z,
      max_z,
      bottom,
      top,
      // The grass would cost the metering much and move its exposure little.
      is_grassed: false,
    }
  }
}
