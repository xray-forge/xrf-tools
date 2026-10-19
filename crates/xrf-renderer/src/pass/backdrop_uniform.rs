use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_page_backdrop::RenderPageBackdrop;
use crate::contract::render_page_plane::RenderPagePlane;

/// The page's backdrop, as `frame/backdrop.wgsl` reads it: WGSL's `Backdrop`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, ShaderStruct)]
#[shader(name = "Backdrop")]
pub struct BackdropUniform {
  /// The wash's first and last colours, each premultiplied by its alpha, as CSS interpolates a gradient.
  pub first: Vec4,
  pub last: Vec4,
  /// The wash's box in window pixels: its left, top, width and height.
  pub rect: Vec4,
  /// The gradient line's direction in window pixels, `y` down, then its length; a zero length paints no wash.
  pub line: Vec4,
  /// Each plane's colour, sRGB from 0 to 1, and one in `w` where the wash is laid over it.
  pub colors: [Vec4; BackdropUniform::PLANES],
  /// Each plane's box in window pixels, as `rect`; the first fills the window, and an empty one paints nothing.
  pub boxes: [Vec4; BackdropUniform::PLANES],
  /// Each plane's corner radii in window pixels: top left, top right, bottom right, bottom left.
  pub radii: [Vec4; BackdropUniform::PLANES],
}

impl BackdropUniform {
  /// Planes `frame/backdrop.wgsl` holds room for; a page painting more keeps its outermost and its innermost ones,
  /// which are what show around a viewport.
  pub const PLANES: usize = 4;

  pub fn new(backdrop: &RenderPageBackdrop) -> Self {
    let mut uniform: Self = Self::default();

    if let Some(wash) = backdrop.wash.filter(|wash| wash.rect.width > 0 && wash.rect.height > 0) {
      let (width, height): (f32, f32) = (wash.rect.width as f32, wash.rect.height as f32);
      let (sin, cos): (f32, f32) = wash.angle.to_radians().sin_cos();
      // CSS's gradient line: through the box's centre at the angle, as long as the box projects onto it.
      let length: f32 = (width * sin).abs() + (height * cos).abs();
      let premultiplied = |[r, g, b, a]: [f32; 4]| Vec4::new(r * a, g * a, b * a, a);

      uniform.first = premultiplied(wash.from);
      uniform.last = premultiplied(wash.to);
      uniform.rect = Vec4::new(wash.rect.x as f32, wash.rect.y as f32, width, height);
      uniform.line = Vec4::new(sin, -cos, length, 0.0);
    }

    let planes: &[RenderPagePlane] = &backdrop.planes;
    let skipped: usize = planes.len().saturating_sub(Self::PLANES);
    let kept = planes.iter().take(1).chain(planes.iter().skip(1 + skipped));

    for (index, plane) in kept.enumerate() {
      let (color, rect) = (plane.color, plane.rect);
      let is_washed: bool = plane.is_washed && uniform.line.z > 0.0;

      uniform.colors[index] = Vec4::new(
        f32::from(color.r) / 255.0,
        f32::from(color.g) / 255.0,
        f32::from(color.b) / 255.0,
        if is_washed { 1.0 } else { 0.0 },
      );

      if rect.width > 0 && rect.height > 0 {
        uniform.boxes[index] = Vec4::new(rect.x as f32, rect.y as f32, rect.width as f32, rect.height as f32);
        uniform.radii[index] = Vec4::from_array(plane.radii).max(Vec4::ZERO);
      }
    }

    uniform
  }

  /// The colour the window is cleared to: the first plane's, black where the page paints none.
  pub fn get_clear(&self) -> wgpu::Color {
    let [r, g, b, _] = self.colors[0].to_array();

    wgpu::Color {
      r: f64::from(r),
      g: f64::from(g),
      b: f64::from(b),
      a: 1.0,
    }
  }

  /// Whether the clear paints it all: one plane, and no wash over it.
  pub fn is_flat(&self) -> bool {
    self.colors[0].w == 0.0 && self.boxes[1..].iter().all(|it| it.z <= 0.0 || it.w <= 0.0)
  }
}
