use crate::host::render_helper_shape::RenderHelperShape;

/// One level object the viewer draws a helper for: what names it to the page, the kind its switch is, how it is drawn,
/// and its shapes where it stands.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderHelperObject {
  /// What a pick of it answers, as the application numbers its objects.
  pub id: u32,
  /// The kind its helper is switched by, a bit of `RenderHelperOptions::kinds`, below 32.
  pub kind: u32,
  /// sRGB from 0 to 1.
  pub color: [f32; 3],
  /// Metres from the camera past which it fades out, the selected aside.
  pub draw_distance: f32,
  /// Its `XFORM` in renderer space, column by column.
  pub transform: [f32; 16],
  pub shapes: Vec<RenderHelperShape>,
}
