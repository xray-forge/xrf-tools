/// A helper's colour as a kind table writes it, `#rrggbb` in sRGB, read as the renderer takes it.
pub(crate) struct HelperColor;

impl HelperColor {
  /// `color` as sRGB floats from 0 to 1, a channel not written as two hex digits read as zero.
  pub(crate) fn parse_rgb(color: &str) -> [f32; 3] {
    let channel = |at: usize| -> f32 {
      color
        .get(at..at + 2)
        .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        .map_or(0.0, |value| f32::from(value) / 255.0)
    };

    [channel(1), channel(3), channel(5)]
  }
}
