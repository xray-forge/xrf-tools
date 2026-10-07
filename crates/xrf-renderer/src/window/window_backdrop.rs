use crate::pass::backdrop_uniform::BackdropUniform;

/// What one window's backdrop is painted with: its uniform, what it holds so an unchanged backdrop writes nothing, and
/// the bind group the backdrop pass binds it through.
pub struct WindowBackdrop {
  pub uniform: wgpu::Buffer,
  pub bind_group: wgpu::BindGroup,
  pub written: Option<BackdropUniform>,
}
