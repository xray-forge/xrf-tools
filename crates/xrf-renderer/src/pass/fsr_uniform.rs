use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::camera::camera_view::CameraView;

/// `cbFSR2` as `shaders/frame/fsr/common.wgsl`'s `Fsr` lays it out.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Fsr")]
pub struct FsrUniform {
  pub render_size: Vec2,
  pub display_size: Vec2,
  /// In FSR's sense, the renderer's jitter turned: a drawn texel `m` stands at `m + 0.5 - jitter`.
  pub jitter: Vec2,
  pub downscale: Vec2,
  /// `fDeviceToViewDepth`: device depth to view depth, `y / (d - x)`, and the projection's inverse scales.
  pub device_to_view: Vec4,
  pub luma_mip_size: Vec2,
  pub jitter_phase_count: f32,
  /// Zero on the first frame after a reset.
  pub frame_index: f32,
}

impl FsrUniform {
  /// The constants for a frame drawn `render` texels across into a viewport `display` pixels across, its texels
  /// sampled `jitter` off their centres, `y` down, through `view`'s lens.
  pub fn new(
    (render, display): ((u32, u32), (u32, u32)),
    jitter: Vec2,
    view: &CameraView,
    (luma_mip, phases): ((u32, u32), u32),
    frame_index: u32,
  ) -> Self {
    let (near, far): (f32, f32) = view.get_depth_range();
    let render: Vec2 = Vec2::new(render.0 as f32, render.1 as f32);
    let display: Vec2 = Vec2::new(display.0 as f32, display.1 as f32);

    Self {
      render_size: render,
      display_size: display,
      jitter: -jitter,
      downscale: render / display,
      // Reversed: `d = n (f - z) / (z (f - n))`, so `z = (n f / (f - n)) / (d + n / (f - n))`.
      device_to_view: Vec4::new(
        -near / (far - near),
        near * far / (far - near),
        1.0 / view.projection.x_axis.x,
        1.0 / view.projection.y_axis.y,
      ),
      luma_mip_size: Vec2::new(luma_mip.0 as f32, luma_mip.1 as f32),
      jitter_phase_count: phases as f32,
      frame_index: frame_index as f32,
    }
  }
}
