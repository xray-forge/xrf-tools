use serde::{Deserialize, Serialize};

/// Map pixels a level map's tile spans on a side at most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAP_TILE: u32 = 2048;

/// Map pixels a level map's tile is drawn past each side, as far as the screen-space passes gather around a pixel.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAP_TILE_MARGIN: u32 = 64;

/// Map pixels a level map spans on a side at most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAP_MOST_PIXELS: u32 = 16384;

/// Times over a level map's tile is drawn at most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAP_MOST_SUPERSAMPLING: u32 = 4;

/// A level map asked of a viewport: an area in engine space seen straight down, `width` by `height` pixels.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMapCapture {
  /// The area's least `x` and `z`, the map's left and bottom edges.
  pub min_x: f32,
  pub min_z: f32,
  /// Its greatest `x` and `z`, the map's right and top edges.
  pub max_x: f32,
  pub max_z: f32,
  /// The heights between which the level is drawn: what stands below `bottom` or above `top` is cut away.
  pub bottom: f32,
  pub top: f32,
  /// The map's pixels across and down.
  pub width: u32,
  pub height: u32,
  /// Times over on each side a tile is drawn before it is reduced to the map's pixels.
  pub supersampling: u32,
  /// Frames each tile is drawn from where it stands before the one kept, for what builds over frames to settle.
  pub settle_frames: u32,
  /// Whether the grass is planted over each tile, which `CDemoRecord`'s map draws none of: an experiment.
  pub is_grassed: bool,
}
