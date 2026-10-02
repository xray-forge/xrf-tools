use std::collections::HashMap;

use wgpu::util::DeviceExt;
use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::lighting::material_lut::{MATERIAL_LUT_DEPTH, MATERIAL_LUT_HEIGHT, MATERIAL_LUT_WIDTH, create_material_lut};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Lights a viewport's G-buffer with the sun and the hemisphere, and tonemaps it into its rectangle of the window.
pub struct CombinePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// One a target format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
  lut: wgpu::TextureView,
  sampler: wgpu::Sampler,
  generation: u64,
}

impl CombinePass {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
  ) -> Self {
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("combine"),
      entries: &[
        texture_entry(0, fragment, unfiltered, flat),
        texture_entry(1, fragment, unfiltered, flat),
        texture_entry(2, fragment, unfiltered, flat),
        texture_entry(3, fragment, wgpu::TextureSampleType::Depth, flat),
        texture_entry(
          4,
          fragment,
          wgpu::TextureSampleType::Float { filterable: true },
          wgpu::TextureViewDimension::D3,
        ),
        wgpu::BindGroupLayoutEntry {
          binding: 5,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        uniform_entry(6, wgpu::ShaderStages::FRAGMENT),
      ],
    });
    let lut: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("material table"),
          size: wgpu::Extent3d {
            width: MATERIAL_LUT_WIDTH,
            height: MATERIAL_LUT_HEIGHT,
            depth_or_array_layers: MATERIAL_LUT_DEPTH,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D3,
          format: wgpu::TextureFormat::Rg8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &create_material_lut(),
      )
      .create_view(&Default::default());
    let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("material table"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });

    Self {
      layout,
      view_layout: view_layout.clone(),
      pipelines: HashMap::new(),
      lut,
      sampler,
      generation: shaders.get_generation(),
    }
  }

  /// Forgets pipelines built from a library reloaded since, so the next draw builds them again.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();
      self.pipelines.clear();
    }
  }

  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    lighting: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("combine"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&targets.albedo),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&targets.normal),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::TextureView(&targets.material),
        },
        wgpu::BindGroupEntry {
          binding: 3,
          resource: wgpu::BindingResource::TextureView(&targets.depth),
        },
        wgpu::BindGroupEntry {
          binding: 4,
          resource: wgpu::BindingResource::TextureView(&self.lut),
        },
        wgpu::BindGroupEntry {
          binding: 5,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
        wgpu::BindGroupEntry {
          binding: 6,
          resource: lighting.as_entire_binding(),
        },
      ],
    })
  }

  /// Builds the pipeline drawing into a format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if self.pipelines.contains_key(&format) {
      return Ok(());
    }

    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/combine")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("combine"),
      bind_group_layouts: &[Some(&self.view_layout), Some(&self.layout)],
      ..Default::default()
    });
    let pipeline: wgpu::RenderPipeline = create_checked(device, "combine", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("combine"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_main"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some("fs_main"),
          compilation_options: Default::default(),
          targets: &[Some(format.into())],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    })?;

    self.pipelines.insert(format, pipeline);

    Ok(())
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
