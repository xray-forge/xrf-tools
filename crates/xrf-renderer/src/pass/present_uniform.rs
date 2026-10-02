use bytemuck::{Pod, Zeroable};

use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;

/// What the present pass shows, as `frame/present.wgsl`'s `Present` reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct PresentUniform {
  /// The debug view's index, zero for the finished scene.
  pub view: u32,
  /// One where the screen's occlusion was searched this frame.
  pub is_occluded: u32,
  /// One where the frame shown is the upscaled one.
  pub is_upscaled: u32,
  pub pad: u32,
  /// The viewport's top left corner in the window, and its size, in pixels.
  pub origin: [f32; 2],
  pub size: [f32; 2],
  /// `img_corrections`' exposure, gamma and saturation, then its grading colour.
  pub corrections: [f32; 4],
  pub grading: [f32; 4],
}

impl PresentUniform {
  pub fn new(
    view: RenderDebugView,
    is_occluded: bool,
    is_upscaled: bool,
    output: RenderRect,
    corrections: &RenderImageCorrections,
  ) -> Self {
    let [r, g, b] = corrections.grading;

    Self {
      corrections: [corrections.exposure, corrections.gamma, corrections.saturation, 0.0],
      grading: [r, g, b, 0.0],
      view: view.get_index(),
      is_occluded: u32::from(is_occluded),
      is_upscaled: u32::from(is_upscaled),
      pad: 0,
      origin: [output.x as f32, output.y as f32],
      size: [output.width as f32, output.height as f32],
    }
  }
}
