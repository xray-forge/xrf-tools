/// A wheel's turn, as the cameras measure it.
pub(crate) struct WheelDelta;

impl WheelDelta {
  /// A line or a page of wheel is worth what a browser scrolls for it, so every mouse moves a camera alike: `height`
  /// is the viewport's in CSS pixels, which a page is.
  pub(crate) fn get_pixels(delta_y: f32, delta_mode: u32, height: f32) -> f32 {
    match delta_mode {
      1 => delta_y * 16.0,
      2 => delta_y * height,
      _ => delta_y,
    }
  }
}
