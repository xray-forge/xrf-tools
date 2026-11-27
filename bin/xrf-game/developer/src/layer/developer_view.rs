/// The window the developer layer draws over, as it stands this frame: its drawable size in device pixels, device
/// pixels per logical unit (egui's points), and device pixels per unit of the window's own coordinates, which mouse
/// positions come in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeveloperView {
  pub width: u32,
  pub height: u32,
  pub scale: f32,
  pub pixel_density: f32,
}

impl DeveloperView {
  /// The least scale the layer draws at, so a window reporting none still lays it out.
  const MIN_SCALE: f32 = 0.5;

  /// Device pixels per point, never below `MIN_SCALE`.
  pub(crate) fn get_scale(&self) -> f32 {
    self.scale.max(Self::MIN_SCALE)
  }

  /// A position in the window's own coordinates, in points.
  pub(crate) fn to_points(self, x: f32, y: f32) -> egui::Pos2 {
    let factor: f32 = self.pixel_density / self.get_scale();

    egui::pos2(x * factor, y * factor)
  }
}
