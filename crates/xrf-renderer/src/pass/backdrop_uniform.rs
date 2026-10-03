use bytemuck::{Pod, Zeroable};

use crate::contract::render_page_backdrop::RenderPageBackdrop;

/// The page's backdrop, as `frame/backdrop.wgsl`'s `Backdrop` reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct BackdropUniform {
  /// The page's colour, sRGB from 0 to 1.
  pub color: [f32; 4],
  /// The wash's first and last colours, each premultiplied by its alpha, as CSS interpolates a gradient.
  pub first: [f32; 4],
  pub last: [f32; 4],
  /// The wash's box in window pixels: its left, top, width and height.
  pub rect: [f32; 4],
  /// The gradient line's direction in window pixels, `y` down, then its length; a zero length paints no wash.
  pub line: [f32; 4],
}

impl BackdropUniform {
  pub fn new(backdrop: &RenderPageBackdrop) -> Self {
    let color = backdrop.color;
    let mut uniform: Self = Self {
      color: [
        f32::from(color.r) / 255.0,
        f32::from(color.g) / 255.0,
        f32::from(color.b) / 255.0,
        1.0,
      ],
      ..Self::default()
    };

    if let Some(wash) = backdrop.wash.filter(|wash| wash.rect.width > 0 && wash.rect.height > 0) {
      let (width, height): (f32, f32) = (wash.rect.width as f32, wash.rect.height as f32);
      let (sin, cos): (f32, f32) = wash.angle.to_radians().sin_cos();
      // CSS's gradient line: through the box's centre at the angle, as long as the box projects onto it.
      let length: f32 = (width * sin).abs() + (height * cos).abs();
      let premultiplied = |[r, g, b, a]: [f32; 4]| [r * a, g * a, b * a, a];

      uniform.first = premultiplied(wash.from);
      uniform.last = premultiplied(wash.to);
      uniform.rect = [wash.rect.x as f32, wash.rect.y as f32, width, height];
      uniform.line = [sin, -cos, length, 0.0];
    }

    uniform
  }

  /// Whether a wash is painted over the colour, which a plain clear cannot draw.
  pub fn is_washed(&self) -> bool {
    self.line[2] > 0.0
  }
}
