use serde::Serialize;

/// Where a texture's typical colour falls against the albedos real, unlit materials have.
///
/// A diffuse texture is an albedo only where nothing is painted into it: under physical light a colour darker than
/// charcoal or brighter than fresh snow is light or shade the artist drew, which the renderer then lights a second time.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AlbedoVerdict {
  /// Darker than any common material: charcoal and fresh asphalt reflect about 0.04.
  Dark,
  /// Within the range unlit materials span.
  Within,
  /// Brighter than any common material: fresh snow reflects about 0.8 to 0.9.
  Bright,
}

impl AlbedoVerdict {
  /// The darkest linear luminance an unlit material plausibly has.
  pub const FLOOR: f32 = 0.03;
  /// The brightest linear luminance an unlit material plausibly has.
  pub const CEILING: f32 = 0.9;

  /// Where a linear luminance falls.
  pub fn of(luminance: f32) -> Self {
    if luminance < Self::FLOOR {
      Self::Dark
    } else if luminance > Self::CEILING {
      Self::Bright
    } else {
      Self::Within
    }
  }
}
