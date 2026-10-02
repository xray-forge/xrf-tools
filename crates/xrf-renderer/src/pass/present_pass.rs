use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::texture_entry;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Puts a viewport's finished scene into its rectangle of the window, dithered to the window's eight bits.
pub struct PresentPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// One a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
  generation: u64,
}

impl PresentPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> Self {
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("present"),
      entries: &[texture_entry(
        0,
        wgpu::ShaderStages::FRAGMENT,
        wgpu::TextureSampleType::Float { filterable: false },
        wgpu::TextureViewDimension::D2,
      )],
    });

    Self {
      layout,
      view_layout: view_layout.clone(),
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

  /// Builds the pipeline drawing into a window format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if !self.pipelines.contains_key(&format) {
      let pipeline: wgpu::RenderPipeline = create_fullscreen_pipeline(
        device,
        shaders,
        "frame/present",
        "fs_present",
        &[Some(&self.view_layout), Some(&self.layout)],
        format,
      )?;

      self.pipelines.insert(format, pipeline);
    }

    Ok(())
  }

  pub fn create_bind_group(&self, device: &wgpu::Device, targets: &ViewTargets) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("present"),
      layout: &self.layout,
      entries: &[texture_binding(0, &targets.scene)],
    })
  }

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already.
  pub fn draw(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    format: wgpu::TextureFormat,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
  ) {
    if let Some(pipeline) = self.pipelines.get(&format) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(0, &view.bind_group, &[]);
      pass.set_bind_group(1, bind_group, &[]);
      pass.draw(0..3, 0..1);
    }
  }
}
