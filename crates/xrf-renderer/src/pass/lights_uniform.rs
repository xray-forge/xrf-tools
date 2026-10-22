use glam::{Mat4, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::camera::camera_view::CameraView;
use crate::contract::render_light_shadow_filter::RenderLightShadowFilter;

/// What the lights are binned and lit by, as `shaders/common/light_clusters.wgsl` declares it: how many stand in view,
/// and the view the clusters cut, its depth sliced exponentially from the near plane to the far one.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Lights")]
pub struct LightsUniform {
  pub count: u32,
  pub near: f32,
  pub far: f32,
  /// How the shadowed lights' maps are compared: zero `shadow_hw`, one `shadow_pcss`.
  pub shadow_filter: u32,
  /// The projection's `x` and `y` scales and its offsets: through a perspective the clip offsets times `w`, through
  /// parallel rays the plain ones.
  pub projection: Vec4,
  /// `x` one where the view's rays are parallel, which a cluster's box spans the same at every depth.
  pub lens: Vec4,
}

impl LightsUniform {
  /// What `count` lights in view are binned and lit by through `camera`.
  pub fn new(count: u32, camera: &CameraView, filter: RenderLightShadowFilter) -> Self {
    let projection: Mat4 = camera.projection;
    let is_orthographic: bool = camera.is_orthographic();
    let (near, far): (f32, f32) = camera.get_depth_range();

    Self {
      count,
      near,
      far,
      shadow_filter: match filter {
        RenderLightShadowFilter::Engine => 0,
        RenderLightShadowFilter::Soft => 1,
      },
      projection: if is_orthographic {
        Vec4::new(
          projection.x_axis.x,
          projection.y_axis.y,
          -projection.w_axis.x,
          -projection.w_axis.y,
        )
      } else {
        Vec4::new(
          projection.x_axis.x,
          projection.y_axis.y,
          projection.z_axis.x,
          projection.z_axis.y,
        )
      },
      lens: Vec4::new(f32::from(u8::from(is_orthographic)), 0.0, 0.0, 0.0),
    }
  }
}
