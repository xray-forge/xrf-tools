use xrf_error::XrfResult;

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::texture_binding;
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Every quality, in the order their search pipelines are held.
const QUALITIES: [RenderAmbientOcclusionQuality; 4] = [
  RenderAmbientOcclusionQuality::Low,
  RenderAmbientOcclusionQuality::Medium,
  RenderAmbientOcclusionQuality::High,
  RenderAmbientOcclusionQuality::Ultra,
];

/// Ambient occlusion at half the frame's size: searched into its first target, denoised across into the second and
/// down back into the first, which combine reads.
pub struct AmbientOcclusionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The search, one a quality in `QUALITIES` order.
  searches: Vec<wgpu::RenderPipeline>,
  /// The denoise across, then down.
  denoises: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl AmbientOcclusionPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("ambient occlusion"),
      entries: &[
        texture_entry(0, fragment, unfiltered, flat),
        texture_entry(1, fragment, wgpu::TextureSampleType::Depth, flat),
        uniform_entry(2, fragment),
        texture_entry(3, fragment, unfiltered, flat),
      ],
    });
    let (searches, denoises) = Self::create_pipelines(device, shaders, view_layout, &layout)?;

    Ok(Self {
      searches,
      denoises,
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout) {
        Ok((searches, denoises)) => {
          self.searches = searches;
          self.denoises = denoises;
        }
        Err(error) => log::error!("Ambient occlusion rejected, occluding with the last one: {error}"),
      }
    }
  }

  /// The two bind groups the passes alternate: each reads the target the other draws into.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    uniform: &wgpu::Buffer,
  ) -> [wgpu::BindGroup; 2] {
    [&targets.occlusion[1], &targets.occlusion[0]].map(|source| {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ambient occlusion"),
        layout: &self.layout,
        entries: &[
          texture_binding(0, &targets.normal),
          texture_binding(1, &targets.depth),
          wgpu::BindGroupEntry {
            binding: 2,
            resource: uniform.as_entire_binding(),
          },
          texture_binding(3, source),
        ],
      })
    })
  }

  /// Which group each stage reads (the other target than it draws into), and which target it draws into: the search,
  /// then the denoise across, then back.
  pub const STAGES: [(usize, usize); 3] = [(0, 0), (1, 1), (0, 0)];

  /// Draws one of its stages into the target the pass draws into: the search at `quality`, or a denoise.
  pub fn record(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    (stage, quality): (usize, RenderAmbientOcclusionQuality),
    view: &ViewBinding,
    bind_groups: &[wgpu::BindGroup; 2],
  ) {
    let search: usize = QUALITIES.iter().position(|it| *it == quality).unwrap_or(2);
    let pipeline: &wgpu::RenderPipeline = match stage {
      0 => &self.searches[search],
      _ => &self.denoises[stage - 1],
    };

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, &bind_groups[Self::STAGES[stage].0], &[]);
    pass.draw(0..3, 0..1);
  }

  #[allow(clippy::type_complexity)]
  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(Vec<wgpu::RenderPipeline>, [wgpu::RenderPipeline; 2])> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/ambient_occlusion")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("ambient occlusion"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    let create = |fragment: &str, constants: &[(&str, f64)]| -> XrfResult<wgpu::RenderPipeline> {
      create_checked(device, "ambient occlusion", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("ambient occlusion"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: wgpu::PipelineCompilationOptions {
              constants,
              ..Default::default()
            },
            targets: &[Some(ViewTargets::OCCLUSION.into())],
          }),
          primitive: Default::default(),
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };
    let mut searches: Vec<wgpu::RenderPipeline> = Vec::with_capacity(QUALITIES.len());

    for quality in QUALITIES {
      let (slices, steps) = quality.get_search();

      searches.push(create(
        "fs_search",
        &[("SLICES", slices as f64), ("STEPS", steps as f64)],
      )?);
    }

    Ok((searches, [create("fs_denoise_x", &[])?, create("fs_denoise_y", &[])?]))
  }
}
