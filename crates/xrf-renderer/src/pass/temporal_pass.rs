use xrf_error::XrfResult;

use crate::frame::temporal_history::TemporalHistory;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{
  begin_cleared_pass, buffer_binding, create_fullscreen_pipeline, texture_binding,
};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Resolves a viewport's jittered frame with its history into the next history, which is the frame shown.
pub struct TemporalPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl TemporalPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("temporal"),
      entries: &[
        texture_entry(0, fragment, wgpu::TextureSampleType::Float { filterable: false }, flat),
        texture_entry(1, fragment, wgpu::TextureSampleType::Depth, flat),
        texture_entry(2, fragment, wgpu::TextureSampleType::Float { filterable: true }, flat),
        wgpu::BindGroupLayoutEntry {
          binding: 3,
          visibility: fragment,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        uniform_entry(4, fragment),
      ],
    });

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("temporal history"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Temporal resolve rejected, resolving with the last one: {error}"),
      }
    }
  }

  /// One bind group a history written: each reads the other.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    history: &TemporalHistory,
    uniform: &wgpu::Buffer,
  ) -> [wgpu::BindGroup; 2] {
    [1, 0].map(|read: usize| {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("temporal"),
        layout: &self.layout,
        entries: &[
          texture_binding(0, &targets.scene),
          texture_binding(1, &targets.depth),
          texture_binding(2, &history.views[read]),
          wgpu::BindGroupEntry {
            binding: 3,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
          buffer_binding(4, uniform),
        ],
      })
    })
  }

  /// Resolves into the history this frame writes.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
    target: &wgpu::TextureView,
  ) {
    let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass(encoder, "temporal", target);

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/temporal",
      "fs_temporal",
      &[Some(view_layout), Some(layout)],
      TemporalHistory::FORMAT,
    )
  }
}
