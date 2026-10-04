use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{
  begin_cleared_pass, buffer_binding, create_fullscreen_pipeline, texture_binding,
};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::shader::shader_library::ShaderLibrary;

/// The engine's bloom (`phase_bloom`): the frame's high part built into the first bloom target, blurred across into the
/// second and down back into the first, which the present adds over the frame.
pub struct BloomPass {
  layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
  build: wgpu::RenderPipeline,
  filter: wgpu::RenderPipeline,
  generation: u64,
}

/// What each of the bloom's three draws reads: the build's, then across's, then down's.
pub struct BloomGroups(pub [wgpu::BindGroup; 3]);

impl BloomPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("bloom"),
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
        uniform_entry(2, fragment),
      ],
    });
    let (build, filter) = Self::create_pipelines(device, shaders, &layout)?;

    Ok(Self {
      build,
      filter,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("bloom"),
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

      match Self::create_pipelines(device, shaders, &self.layout) {
        Ok((build, filter)) => (self.build, self.filter) = (build, filter),
        Err(error) => log::error!("Bloom rejected, blooming with the last one: {error}"),
      }
    }
  }

  /// What the three draws read: the high part, then each bloom target in turn, each with its own uniform.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    uniforms: &[wgpu::Buffer; 3],
  ) -> BloomGroups {
    let group = |label: &str, source: &wgpu::TextureView, uniform: &wgpu::Buffer| {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &self.layout,
        entries: &[
          texture_binding(0, source),
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
          buffer_binding(2, uniform),
        ],
      })
    };

    BloomGroups([
      group("bloom build", &targets.high, &uniforms[0]),
      group("bloom across", &targets.bloom[0], &uniforms[1]),
      group("bloom down", &targets.bloom[1], &uniforms[2]),
    ])
  }

  /// Builds the bloom into the first target, blurs it across into the second and down back into the first.
  pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, targets: &ViewTargets, groups: &BloomGroups) {
    let [build, across, down] = &groups.0;
    let draws: [(&str, &wgpu::RenderPipeline, &wgpu::BindGroup, &wgpu::TextureView); 3] = [
      ("bloom build", &self.build, build, &targets.bloom[0]),
      ("bloom across", &self.filter, across, &targets.bloom[1]),
      ("bloom down", &self.filter, down, &targets.bloom[0]),
    ];

    for (label, pipeline, group, target) in draws {
      let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass(encoder, label, target);

      pass.set_pipeline(pipeline);
      pass.set_bind_group(0, group, &[]);
      pass.draw(0..3, 0..1);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(wgpu::RenderPipeline, wgpu::RenderPipeline)> {
    let create = |fragment: &str| {
      create_fullscreen_pipeline(
        device,
        shaders,
        "frame/bloom",
        fragment,
        &[Some(layout)],
        ViewTargets::BLOOM,
      )
    };

    Ok((create("fs_bloom_build")?, create("fs_bloom_filter")?))
  }
}
