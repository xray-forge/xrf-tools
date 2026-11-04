use glam::Vec3;
use xrf_renderer::{RenderViewOptions, RenderViewportId};

use crate::contract::world_toggles::WorldToggles;
use crate::level::ambient::ambient_frame::AmbientFrame;

/// What the viewport driving a level's scene gives the level's frame: which it is, its view's options and toggles, the
/// ambients its weather plays, none while nothing plays, and where its camera stands, in engine space.
#[derive(Clone, Copy)]
pub struct LevelDrive<'a> {
  pub viewport: RenderViewportId,
  pub options: &'a RenderViewOptions,
  pub toggles: &'a WorldToggles,
  pub ambient: Option<AmbientFrame<'a>>,
  pub eye: Vec3,
}
