/// How a received setting is held to the bounds its schema describes, for what `RenderSettingSchema`'s derive
/// generates: the same rule the page holds a stored value to.
pub(crate) struct RenderSettingBounds;

impl RenderSettingBounds {
  /// A number held between its bounds, and whole where it takes whole values; one that is not a number is the least.
  pub(crate) fn hold_number(value: f32, min: f32, max: f32, is_integer: bool) -> f32 {
    if value.is_nan() {
      return min;
    }

    let held: f32 = value.clamp(min, max);

    if is_integer { held.round() } else { held }
  }

  /// A run of widths cut to the most it holds, each held between its bounds.
  pub(crate) fn hold_widths(widths: &mut Vec<f32>, min: f32, max: f32, most: usize) {
    widths.truncate(most);

    for width in widths.iter_mut() {
      *width = Self::hold_number(*width, min, max, false);
    }
  }
}
