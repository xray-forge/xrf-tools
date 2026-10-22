use std::sync::Arc;

use crate::camera::camera_view::CameraView;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_view_options::RenderViewOptions;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_lighting::RenderLighting;

/// What one view of a scene is drawn through this frame: a viewport's own, or a level map's tile in its place.
pub struct SceneViewFraming<'a> {
  /// Its camera, unjittered.
  pub view: CameraView,
  /// The whole rectangle it is resolved over, and the sizes it is resolved and drawn at.
  pub rect: RenderRect,
  pub resolved: (u32, u32),
  pub drawn: RenderRect,
  pub options: &'a RenderViewOptions,
  pub lighting: &'a RenderLighting,
  pub weather: Option<&'a Arc<RenderLevelWeather>>,
  /// What it draws over its frame and their version, and what of its level is selected; none for a map's tile.
  pub overlays: Option<(&'a [RenderOverlay], u64)>,
  pub selection: Option<&'a RenderSelection>,
  /// How fast the weather's clock runs against real time.
  pub weather_rate: f32,
}
