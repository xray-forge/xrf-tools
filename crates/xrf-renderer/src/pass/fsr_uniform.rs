use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::camera::camera_view::CameraView;
use crate::pass::fsr_frame::FsrFrame;

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
  /// `fDeltaPreExposure`: this frame's pre-exposure over the last's, which the reprojected history is multiplied by.
  pub delta_pre_exposure: f32,
  pub _pad: [f32; 3],
}

impl FsrUniform {
  /// AMD's default `fVelocityFactor`: the history's weight falls to its least at half a 4K pixel of motion.
  pub const VELOCITY_FACTOR: f32 = 1.0;

  /// The constants for a `frame`, its jitter `y` down, seen through `view`'s lens.
  pub fn new(frame: FsrFrame, view: &CameraView) -> Self {
    let (near, far): (f32, f32) = view.get_depth_range();
    let shading: (u32, u32) = Self::get_shading_size(frame.render);
    let render: Vec2 = Vec2::new(frame.render.0 as f32, frame.render.1 as f32);
    let display: Vec2 = Vec2::new(frame.display.0 as f32, frame.display.1 as f32);

    Self {
      render_size: render,
      display_size: display,
      jitter: -frame.jitter,
      previous_jitter: -frame.previous_jitter,
      downscale: render / display,
      shading_size: Vec2::new(shading.0 as f32, shading.1 as f32),
      // Reversed: `d = n (f - z) / (z (f - n))`, so `z = (n f / (f - n)) / (d + n / (f - n))`.
      device_to_view: Vec4::new(
        -near / (far - near),
        near * far / (far - near),
        1.0 / view.projection.x_axis.x,
        1.0 / view.projection.y_axis.y,
      ),
      jitter_phase_count: frame.phases as f32,
      frame_index: frame.frame_index as f32,
      velocity_factor: Self::VELOCITY_FACTOR,
      hdr: f32::from(u8::from(frame.is_hdr)),
      delta_pre_exposure: 1.0,
      _pad: [0.0; 3],
    }
  }

  /// The same over colour written at a pre-exposure `delta` times the last frame's.
  pub fn with_delta_pre_exposure(self, delta: f32) -> Self {
    Self {
      delta_pre_exposure: delta,
      ..self
    }
  }

  /// The shading change's size for a drawing: half of it, truncated, never none.
  pub fn get_shading_size((width, height): (u32, u32)) -> (u32, u32) {
    ((width / 2).max(1), (height / 2).max(1))
  }
}
