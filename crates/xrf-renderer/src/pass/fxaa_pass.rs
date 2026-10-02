use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{begin_cleared_pass, create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::texture_entry;
use crate::shader::shader_library::ShaderLibrary;

/// FXAA over a viewport's scene as drawn, into a target the scene is copied back from.
pub struct FxaaPass {
  layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl FxaaPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("fxaa"),
      entries: &[
        texture_entry(
          0,
          fragment,
          wgpu::TextureSampleType::Float { filterable: true },
          wgpu::TextureViewDimension::D2,
        ),
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: fragment,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fxaa"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("FXAA rejected, smoothing with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_group(&self, device: &wgpu::Device, targets: &ViewTargets) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("fxaa"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.scene),
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
      ],
    })
  }

  pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, bind_group: &wgpu::BindGroup, target: &wgpu::TextureView) {
    let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass(encoder, "fxaa", target);

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/fxaa",
      "fs_fxaa",
      &[Some(layout)],
      ViewTargets::SCENE,
    )
  }
}
