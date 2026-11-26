/// A helper's `#rrggbb` sRGB colour from a kind table, read for the renderer.
pub struct HelperColor;

impl HelperColor {
  /// `color` as sRGB floats from 0 to 1; a channel that is not two hex digits reads as zero.
  pub fn parse_rgb(color: &str) -> [f32; 3] {
    let channel = |at: usize| -> f32 {
      color
        .get(at..at + 2)
        .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        .map_or(0.0, |value| f32::from(value) / 255.0)
    };

    [channel(1), channel(3), channel(5)]
  }
}
