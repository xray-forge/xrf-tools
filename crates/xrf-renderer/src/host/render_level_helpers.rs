use crate::host::render_helper_object::RenderHelperObject;

/// The level objects a level's viewer draws helpers for: editor-only pictures of what draws nothing of its own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderLevelHelpers {
  pub objects: Vec<RenderHelperObject>,
}
