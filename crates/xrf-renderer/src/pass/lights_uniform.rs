use glam::{Mat4, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_light_shadow_filter::RenderLightShadowFilter;

/// What the lights are binned and lit by, as `shaders/common/light_clusters.wgsl` declares it: how many stand in view, and the
/// view the clusters cut, its depth sliced exponentially from the near plane to the far one.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Lights")]
pub struct LightsUniform {
  pub count: u32,
  pub near: f32,
  pub far: f32,
  /// How the shadowed lights' maps are compared: zero `shadow_hw`, one `shadow_pcss`.
  pub shadow_filter: u32,
  /// The projection's `x` and `y` scales and its offsets.
  pub projection: Vec4,
}

impl LightsUniform {
  pub fn new(count: u32, projection: Mat4, near: f32, far: f32, filter: RenderLightShadowFilter) -> Self {
    Self {
      count,
      near,
      far,
      shadow_filter: match filter {
        RenderLightShadowFilter::Engine => 0,
        RenderLightShadowFilter::Soft => 1,
      },
      projection: Vec4::new(
        projection.x_axis.x,
        projection.y_axis.y,
        projection.z_axis.x,
        projection.z_axis.y,
      ),
    }
  }
}
