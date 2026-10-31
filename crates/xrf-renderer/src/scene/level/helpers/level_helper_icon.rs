use glam::Vec3;

use crate::contract::render_helper_icon::RenderHelperIcon;

/// One icon a helped object stands, as a scene keeps it: where it stands, renderer space, and its picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelHelperIcon {
  pub center: Vec3,
  pub icon: RenderHelperIcon,
}
