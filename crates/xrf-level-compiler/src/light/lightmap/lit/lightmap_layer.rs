use crate::light::point::light_color::LightColor;

/// A rectangle of lightmap texels, `lm_layer` (`xrLC_Light/lm_layer.h`): each texel's light as `base_color` stores it
/// ([`LightColor::get_stored`]) and its marker, 0 for a texel nothing has reached, 255 for one a face lit, and less for
/// one a border ring filled, the further out the less.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LightmapLayer {
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) surface: Vec<LightColor>,
  pub(crate) marker: Vec<u8>,
}

impl LightmapLayer {
  /// A marker of a texel a face lit.
  pub(crate) const LIT: u8 = 255;

  /// The least marker a texel a chart takes on a page has, `254 - BORDER`: its lit texels and two rings.
  pub(crate) const TAKEN: u8 = 253;

  /// A layer `width` by `height` of no light, nothing marked.
  pub(crate) fn new(width: u32, height: u32) -> Self {
    let count: usize = (width * height) as usize;

    Self {
      width,
      height,
      surface: vec![LightColor::default(); count],
      marker: vec![0; count],
    }
  }

  /// A layer `width` by `height` every texel of `color`, lit.
  pub(crate) fn filled(width: u32, height: u32, color: LightColor) -> Self {
    let count: usize = (width * height) as usize;

    Self {
      width,
      height,
      surface: vec![color; count],
      marker: vec![Self::LIT; count],
    }
  }

  /// `ApplyBorders`: every texel nothing has reached that has a neighbour, beside it or across a corner, marked above
  /// `reference` takes those neighbours' mean light, as stored, and `reference`; all at once, as the source's scan
  /// works out to. Whether any texel was reached.
  pub(crate) fn apply_borders(&mut self, reference: u8) -> bool {
    let (width, height) = (i64::from(self.width), i64::from(self.height));
    let (surface, marker) = (self.surface.clone(), self.marker.clone());
    let mut is_reached: bool = false;

    for y in 0..height {
      for x in 0..width {
        let index: usize = (y * width + x) as usize;

        if marker[index] != 0 {
          continue;
        }

        let mut sum: LightColor = LightColor::default();
        let mut count: u32 = 0;

        for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
          let (nx, ny) = (x + dx, y + dy);

          if !(0..width).contains(&nx) || !(0..height).contains(&ny) {
            continue;
          }

          let neighbour: usize = (ny * width + nx) as usize;

          if marker[neighbour] > reference {
            let color: &LightColor = &surface[neighbour];

            sum = LightColor {
              rgb: sum.rgb + color.rgb,
              hemi: sum.hemi + color.hemi,
              sun: sum.sun + color.sun,
            };
            count += 1;
          }
        }

        if count > 0 {
          self.surface[index] = sum.get_scaled(1.0 / count as f32).get_stored();
          self.marker[index] = reference;
          is_reached = true;
        }
      }
    }

    is_reached
  }

  /// `ApplyBorders` from `first` down while it reaches a texel, to 1 at the least.
  pub(crate) fn fill_borders(&mut self, first: u8) {
    for reference in (1..=first).rev() {
      if !self.apply_borders(reference) {
        break;
      }
    }
  }

  /// `lm_layer::Pack`: each texel's static light and sun as bytes, red, green, blue and sun.
  pub(crate) fn get_packed(&self) -> Vec<[u8; 4]> {
    self
      .surface
      .iter()
      .map(|color| {
        [
          LightColor::get_byte(color.rgb.x),
          LightColor::get_byte(color.rgb.y),
          LightColor::get_byte(color.rgb.z),
          LightColor::get_byte(color.sun),
        ]
      })
      .collect()
  }

  /// `lm_layer::Pack_hemi`: each texel's sun in every colour and its hemisphere in alpha, as bytes.
  pub(crate) fn get_packed_hemi(&self) -> Vec<[u8; 4]> {
    self
      .surface
      .iter()
      .map(|color| {
        let sun: u8 = LightColor::get_byte(color.sun);

        [sun, sun, sun, LightColor::get_byte(color.hemi)]
      })
      .collect()
  }

  /// The texel at `x`, `y`.
  pub(crate) fn get_index(&self, x: u32, y: u32) -> usize {
    (y * self.width + x) as usize
  }
}
