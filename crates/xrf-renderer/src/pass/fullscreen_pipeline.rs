use xrf_error::XrfResult;

use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::shader::shader_library::ShaderLibrary;

/// A pass drawing one triangle over its whole target, its vertices from `common/fullscreen`, into one target: a format,
/// or a format with a blend.
pub fn create_fullscreen_pipeline(
  device: &wgpu::Device,
  shaders: &ShaderLibrary,
  module: &str,
  fragment: &str,
  layouts: &[Option<&wgpu::BindGroupLayout>],
  target: impl Into<wgpu::ColorTargetState>,
) -> XrfResult<wgpu::RenderPipeline> {
  let target: wgpu::ColorTargetState = target.into();

  let shader: wgpu::ShaderModule = create_module(device, shaders, module)?;
  let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some(module),
    bind_group_layouts: layouts,
    ..Default::default()
  });

  create_checked(device, module, || {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some(module),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_fullscreen"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some(fragment),
        compilation_options: Default::default(),
        targets: &[Some(target.clone())],
      }),
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      multiview_mask: None,
      cache: None,
    })
  })
}

/// One texture view bound at a binding.
pub fn texture_binding(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
  wgpu::BindGroupEntry {
    binding,
    resource: wgpu::BindingResource::TextureView(view),
  }
}

/// One buffer bound whole at a binding.
pub fn buffer_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
  wgpu::BindGroupEntry {
    binding,
    resource: buffer.as_entire_binding(),
  }
}

/// Begins a render pass drawing into one colour target alone, cleared to nothing first.
pub fn begin_cleared_pass<'a>(
  encoder: &'a mut wgpu::CommandEncoder,
  label: &str,
  target: &'a wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
  encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    label: Some(label),
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
      view: target,
      depth_slice: None,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        store: wgpu::StoreOp::Store,
      },
    })],
    ..Default::default()
  })
}
