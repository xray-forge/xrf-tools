use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::camera::camera_view::CameraView;

/// `cbFSR3Upscaler` as `shaders/frame/fsr/common.wgsl`'s `Fsr` lays it out.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Fsr")]
pub struct FsrUniform {
  pub render_size: Vec2,
  pub display_size: Vec2,
  /// In FSR's sense, the renderer's jitter turned: a drawn texel `m` stands at `m + 0.5 - jitter`.
  pub jitter: Vec2,
  pub previous_jitter: Vec2,
  pub downscale: Vec2,
  /// The shading change's size, half the drawn size: `ShadingChangeRenderSize`.
  pub shading_size: Vec2,
  /// `fDeviceToViewDepth`: device depth to view depth, `y / (d - x)`, and the projection's inverse scales.
  pub device_to_view: Vec4,
  pub jitter_phase_count: f32,
  /// Zero on the first frame after a reset.
  pub frame_index: f32,
  /// `fVelocityFactor`: how far the view's motion cuts the history's weight.
  pub velocity_factor: f32,
  /// One where the colour is linear light past the display's range: `FFX_FSR3UPSCALER_OPTION_HDR_COLOR_INPUT`.
  pub hdr: f32,
}

impl FsrUniform {
  /// AMD's default `fVelocityFactor`: the history's weight falls to its least at half a 4K pixel of motion.
  pub const VELOCITY_FACTOR: f32 = 1.0;

  /// The constants for a frame drawn `render` texels across into a viewport `display` pixels across, its texels
  /// sampled `jitter` off their centres, `y` down, through `view`'s lens; `previous_jitter` the frame before's.
  pub fn new(
    (render, display): ((u32, u32), (u32, u32)),
    (jitter, previous_jitter): (Vec2, Vec2),
    view: &CameraView,
    (phases, frame_index): (u32, u32),
  ) -> Self {
    let (near, far): (f32, f32) = view.get_depth_range();
    let shading: (u32, u32) = Self::get_shading_size(render);
    let render: Vec2 = Vec2::new(render.0 as f32, render.1 as f32);
    let display: Vec2 = Vec2::new(display.0 as f32, display.1 as f32);

    Self {
      render_size: render,
      display_size: display,
      jitter: -jitter,
      previous_jitter: -previous_jitter,
      downscale: render / display,
      shading_size: Vec2::new(shading.0 as f32, shading.1 as f32),
      // Reversed: `d = n (f - z) / (z (f - n))`, so `z = (n f / (f - n)) / (d + n / (f - n))`.
      device_to_view: Vec4::new(
        -near / (far - near),
        near * far / (far - near),
        1.0 / view.projection.x_axis.x,
        1.0 / view.projection.y_axis.y,
      ),
      jitter_phase_count: phases as f32,
      frame_index: frame_index as f32,
      velocity_factor: Self::VELOCITY_FACTOR,
      hdr: 0.0,
    }
  }

  /// The same over colour of linear light (`is_hdr`), which it accumulates through its reversible tonemap.
  pub fn with_hdr(self, is_hdr: bool) -> Self {
    Self {
      hdr: f32::from(u8::from(is_hdr)),
      ..self
    }
  }

  /// The shading change's size for a drawing: half of it, truncated, never none.
  pub fn get_shading_size((width, height): (u32, u32)) -> (u32, u32) {
    ((width / 2).max(1), (height / 2).max(1))
  }
}
