use std::borrow::Cow;

use crate::camera::camera_view::CameraView;
use crate::capture::cube_capture_look::CubeCaptureLook;
use crate::capture::cube_face::CubeFace;
use crate::capture::map_capture_look::MapCaptureLook;
use crate::capture::map_tile::MapTile;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;

/// One view a capture draws in place of its viewport's frames: a level map's tile, or a cubemap's face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureShot {
  Tile(MapTile),
  Face(CubeFace),
}

impl CaptureShot {
  /// The pixels it is drawn at, across and down.
  pub fn get_drawn_size(&self) -> (u32, u32) {
    match self {
      Self::Tile(tile) => tile.get_drawn_size(),
      Self::Face(face) => face.get_drawn_size(),
    }
  }

  /// Its view, renderer space, unjittered.
  pub fn get_view(&self) -> CameraView {
    match self {
      Self::Tile(tile) => tile.get_view(),
      Self::Face(face) => face.get_view(),
    }
  }

  /// Whether it is read back with its coverage in alpha, as a map is, rather than opaque.
  pub fn is_covered(&self) -> bool {
    matches!(self, Self::Tile(_))
  }

  /// The options it is drawn with, from the viewport's `options`; metering, its exposure adapts as its kind meters.
  pub fn apply_look(&self, options: &RenderViewOptions, is_metering: bool) -> RenderViewOptions {
    match (self, is_metering) {
      (Self::Tile(tile), false) => MapCaptureLook::apply(options, tile),
      (Self::Tile(tile), true) => MapCaptureLook::apply_metering(options, tile),
      (Self::Face(_), false) => CubeCaptureLook::apply(options),
      (Self::Face(_), true) => CubeCaptureLook::apply_metering(options),
    }
  }

  /// The lighting it is drawn under, from the viewport's.
  pub fn apply_lighting<'a>(&self, lighting: Cow<'a, RenderLighting>) -> Cow<'a, RenderLighting> {
    match self {
      Self::Tile(_) => Cow::Owned(MapCaptureLook::apply_lighting(&lighting)),
      Self::Face(_) => lighting,
    }
  }
}
