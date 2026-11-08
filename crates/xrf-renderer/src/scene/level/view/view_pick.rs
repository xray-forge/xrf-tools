use crate::pass::view::camera_uniform::CameraUniform;

/// A pick a frame readies: its camera narrowed to the texel picked, the layout the view binds by, and whether it picks
/// through glass and the other composited surfaces.
#[derive(Clone, Copy)]
pub(crate) struct ViewPick<'a> {
  pub camera: &'a CameraUniform,
  pub view_layout: &'a wgpu::BindGroupLayout,
  pub is_through: bool,
}
