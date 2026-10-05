use xrf_error::XrfResult;

use crate::frame::upscale_targets::UpscaleTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::shader::shader_library::ShaderLibrary;

/// FSR 1 over a frame drawn smaller than its viewport: EASU upscales it, RCAS sharpens the upscaled frame.
pub struct UpscalePass {
  layout: wgpu::BindGroupLayout,
  /// EASU's, then RCAS's.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl UpscalePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("upscale"),
      entries: &[
        texture_entry(
          0,
          fragment,
          wgpu::TextureSampleType::Float { filterable: false },
          wgpu::TextureViewDimension::D2,
        ),
        uniform_entry(1, fragment),
      ],
    });

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layout)?,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Upscaling rejected, upscaling with the last one: {error}"),
      }
    }
  }

  /// Binds what a pass reads from: the scene for EASU, the upscaled frame for RCAS.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    source: &wgpu::TextureView,
    uniform: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("upscale"),
      layout: &self.layout,
      entries: &[texture_binding(0, source), buffer_binding(1, uniform)],
    })
  }

  /// Draws EASU's upscale (`stage` 0) or RCAS's sharpening (1) into the target the pass draws into.
  pub fn record(&self, pass: &mut wgpu::RenderPass<'_>, stage: usize, group: &wgpu::BindGroup) {
    pass.set_pipeline(&self.pipelines[stage]);
    pass.set_bind_group(0, group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let create = |fragment: &str| {
      create_fullscreen_pipeline(
        device,
        shaders,
        "frame/upscale",
        fragment,
        &[Some(layout)],
        UpscaleTargets::FORMAT,
      )
    };

    Ok([create("fs_easu")?, create("fs_rcas")?])
  }
}
