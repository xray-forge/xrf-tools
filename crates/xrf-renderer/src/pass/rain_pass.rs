use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::rain_bindings::RainBindings;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Draws the rain over a viewport's finished scene as `RenderLast` does after the forward surfaces: the streaks, then
/// the splashes, blended by their alpha and tested against the G-buffer's depth without writing it.
pub struct RainPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// The streaks', then the splashes'.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl RainPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let stages: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("rain"),
      entries: &[
        uniform_entry(0, stages),
        texture_entry(1, stages, wgpu::TextureSampleType::Depth, flat),
        texture_entry(2, stages, filtered, flat),
        texture_entry(3, stages, filtered, flat),
        wgpu::BindGroupLayoutEntry {
          binding: 4,
          visibility: stages,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        storage_entry(5, stages, false),
        storage_entry(6, stages, false),
      ],
    });

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("rain"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
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

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Rain rejected, falling with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_group(&self, device: &wgpu::Device, bindings: &RainBindings<'_>) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("rain"),
      layout: &self.layout,
      entries: &[
        buffer_binding(0, bindings.uniform),
        texture_binding(1, bindings.cover),
        texture_binding(2, bindings.streak),
        texture_binding(3, bindings.splash),
        wgpu::BindGroupEntry {
          binding: 4,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
        buffer_binding(5, bindings.vertices),
        buffer_binding(6, bindings.indices),
      ],
    })
  }

  /// Draws the streaks falling, and the splashes where the model has indices.
  pub fn record(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
    (streaks, splash_indices): (u32, u32),
  ) {
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);
    pass.set_pipeline(&self.pipelines[0]);
    pass.draw(0..streaks * 6, 0..1);

    if splash_indices > 0 {
      pass.set_pipeline(&self.pipelines[1]);
      pass.draw(0..streaks * splash_indices, 0..1);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/rain")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("rain"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    // `blend(true, srcalpha, invsrcalpha)`, the alpha kept under it.
    let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
      format: ViewTargets::SCENE,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::SrcAlpha,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::One,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
      }),
      write_mask: wgpu::ColorWrites::ALL,
    })];
    let create = |vertex: &str, fragment: &str| {
      create_checked(device, "rain", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("rain"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some(vertex),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets: &targets,
          }),
          // Both sides, as the engine's `CULL_NONE`.
          primitive: Default::default(),
          depth_stencil: Some(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: Default::default(),
            bias: Default::default(),
          }),
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };

    Ok([create("vs_streak", "fs_streak")?, create("vs_splash", "fs_splash")?])
  }
}
