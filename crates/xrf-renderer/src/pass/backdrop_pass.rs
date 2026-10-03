use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline};
use crate::pass::layout_entries::uniform_entry;
use crate::shader::shader_library::ShaderLibrary;

/// Paints the page's backdrop over a window where it is transparent, under its viewports: the colour the page shows
/// there, and its wash.
pub struct BackdropPass {
  layout: wgpu::BindGroupLayout,
  uniform: wgpu::Buffer,
  bind_group: wgpu::BindGroup,
  /// What the uniform holds, so an unchanged backdrop writes nothing.
  written: Option<BackdropUniform>,
  /// One a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
  generation: u64,
}

impl BackdropPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> Self {
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("backdrop"),
      entries: &[uniform_entry(0, wgpu::ShaderStages::FRAGMENT)],
    });
    let uniform: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("backdrop"),
      size: size_of::<BackdropUniform>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("backdrop"),
      layout: &layout,
      entries: &[buffer_binding(0, &uniform)],
    });

    Self {
      layout,
      uniform,
      bind_group,
      written: None,
      pipelines: HashMap::new(),
      generation: shaders.get_generation(),
    }
  }

  /// Forgets pipelines built from a library reloaded since, so the next frame builds them again.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();
      self.pipelines.clear();
    }
  }

  /// Builds the pipeline drawing into a window format, unless it is built, and writes the backdrop where it changed.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    format: wgpu::TextureFormat,
    backdrop: &BackdropUniform,
  ) -> XrfResult {
    if self.written.as_ref() != Some(backdrop) {
      queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(backdrop));
      self.written = Some(*backdrop);
    }

    if !self.pipelines.contains_key(&format) {
      let pipeline: wgpu::RenderPipeline = create_fullscreen_pipeline(
        device,
        shaders,
        "frame/backdrop",
        "fs_backdrop",
        &[Some(&self.layout)],
        format,
      )?;

      self.pipelines.insert(format, pipeline);
    }

    Ok(())
  }

  /// Paints the backdrop over the whole window, before its viewports are drawn.
  pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, format: wgpu::TextureFormat) {
    if let Some(pipeline) = self.pipelines.get(&format) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(0, &self.bind_group, &[]);
      pass.draw(0..3, 0..1);
    }
  }
}
