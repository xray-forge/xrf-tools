use crate::contract::settings::schema::render_setting_field::RenderSettingField;
use crate::contract::settings::schema::render_setting_node::RenderSettingNode;

/// A type the settings describe for the page: a group of settings, a choice, or a flag. Derived
/// (`xrf_renderer_derive::RenderSettingSchema`) where each field's bounds stand beside it; a number is described by
/// the bounds on its field, never by its type.
pub(crate) trait RenderSettingSchema {
  fn describe() -> RenderSettingNode;

  /// Hold every setting to the bounds [`Self::describe`] gives it, as the page holds what it stores, so a value sent
  /// from elsewhere is drawn as the page would draw it. A flag or a choice is held by its type already, and what the
  /// settings do not set is left as it is.
  fn hold_to_bounds(&mut self) {}
}

impl RenderSettingSchema for bool {
  fn describe() -> RenderSettingNode {
    RenderSettingNode::Field(RenderSettingField::Flag)
  }
}
