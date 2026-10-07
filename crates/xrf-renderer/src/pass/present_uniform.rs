use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;

/// What the present pass shows and the overlays drawn over it read, as WGSL's `Present`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Present")]
pub struct PresentUniform {
  /// The debug view's index, zero for the finished scene.
  pub view: u32,
  /// One where the screen's occlusion was searched this frame.
  pub is_occluded: u32,
  /// One where the frame shown is the upscaled one.
  pub is_upscaled: u32,
  /// How far the distortion target moves the scene, a share of the screen; zero where nothing wrote it this frame.
  pub distortion: f32,
  /// The viewport's top left corner in the window, and its size, in pixels.
  pub origin: Vec2,
  pub size: Vec2,
  /// `img_corrections`' exposure, gamma and saturation, then its grading colour.
  pub corrections: Vec4,
  pub grading: Vec4,
  /// The colour a selection is outlined in, `w` one while something is selected.
  pub selection: Vec4,
  /// One where the frame blooms, which the present adds where it reads the scene.
  pub is_bloomed: u32,
  pub _pad: [u32; 3],
}

impl PresentUniform {
  pub fn new(
    view: RenderDebugView,
    is_occluded: bool,
    is_upscaled: bool,
    distortion: f32,
    output: RenderRect,
    corrections: &RenderImageCorrections,
    (selection, is_bloomed): (Option<[f32; 3]>, bool),
  ) -> Self {
    let [r, g, b] = corrections.grading;

    Self {
      corrections: Vec4::new(corrections.exposure, corrections.gamma, corrections.saturation, 0.0),
      grading: Vec4::new(r, g, b, 0.0),
      view: view.get_index(),
      is_occluded: u32::from(is_occluded),
      is_upscaled: u32::from(is_upscaled),
      distortion,
      origin: Vec2::new(output.x as f32, output.y as f32),
      size: Vec2::new(output.width as f32, output.height as f32),
      selection: selection.map_or(Vec4::ZERO, |[r, g, b]| Vec4::new(r, g, b, 1.0)),
      is_bloomed: u32::from(is_bloomed),
      _pad: [0; 3],
    }
  }
}
