/// FSR 2's bind groups for one viewport's targets: one a stage, and one a frame for the stages that read the other
/// frame's targets.
pub struct FsrGroups {
  pub luma_first: wgpu::BindGroup,
  pub luma_shading: wgpu::BindGroup,
  pub reconstruct: wgpu::BindGroup,
  pub dilate: wgpu::BindGroup,
  pub reactive: wgpu::BindGroup,
  pub depth_clip: [wgpu::BindGroup; 2],
  pub lock: [wgpu::BindGroup; 2],
  pub accumulate: [wgpu::BindGroup; 2],
}
