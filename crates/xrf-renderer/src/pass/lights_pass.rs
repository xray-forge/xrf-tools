use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::light_buffers::LightBuffers;
use crate::pass::material_table::MaterialTable;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Clusters the view is cut into, and invocations a binning workgroup runs, as the shaders declare them.
const LIGHT_CLUSTERS: u32 = 16 * 9 * 24;
const BINNING_WORKGROUP: u32 = 64;

/// Bins the local lights in view into clusters of the view, then adds every light reaching each pixel to the light its
/// frame accumulated after the sun.
pub struct LightsPass {
  binning_layout: wgpu::BindGroupLayout,
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  binning: wgpu::ComputePipeline,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl LightsPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let compute: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let binning_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("light binning"),
      entries: &[
        storage_entry(0, compute, false),
        storage_entry(1, compute, true),
        storage_entry(2, compute, true),
        uniform_entry(3, compute),
      ],
    });
    let [table, sampler] = MaterialTable::get_layout_entries(3);
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("lights"),
      entries: &[
        texture_entry(0, fragment, unfiltered, flat),
        texture_entry(1, fragment, unfiltered, flat),
        texture_entry(2, fragment, wgpu::TextureSampleType::Depth, flat),
        table,
        sampler,
        storage_entry(5, fragment, false),
        storage_entry(6, fragment, false),
        storage_entry(7, fragment, false),
        uniform_entry(8, fragment),
        texture_entry(9, fragment, wgpu::TextureSampleType::Depth, flat),
      ],
    });
    let (binning, pipeline) =
      Self::create_pipelines(device, shaders, &binning_layout, view_layout, &layout, texture_layout)?;

    Ok(Self {
      binning,
      pipeline,
      binning_layout,
      layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(
        device,
        shaders,
        &self.binning_layout,
        &self.view_layout,
        &self.layout,
        &self.texture_layout,
      ) {
        Ok((binning, pipeline)) => {
          self.binning = binning;
          self.pipeline = pipeline;
        }
        Err(error) => log::error!("Lights rejected, lighting with the last one: {error}"),
      }
    }
  }

  /// The binning's bind group, then the shading's.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    table: &MaterialTable,
    buffers: &LightBuffers<'_>,
    atlas: &wgpu::TextureView,
  ) -> [wgpu::BindGroup; 2] {
    let [table, sampler] = table.get_entries(3);

    [
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("light binning"),
        layout: &self.binning_layout,
        entries: &[
          buffer_binding(0, buffers.records),
          buffer_binding(1, buffers.counts),
          buffer_binding(2, buffers.items),
          buffer_binding(3, buffers.uniform),
        ],
      }),
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("lights"),
        layout: &self.layout,
        entries: &[
          texture_binding(0, &targets.normal),
          texture_binding(1, &targets.material),
          texture_binding(2, &targets.depth),
          table,
          sampler,
          buffer_binding(5, buffers.records),
          buffer_binding(6, buffers.counts),
          buffer_binding(7, buffers.items),
          buffer_binding(8, buffers.uniform),
          texture_binding(9, atlas),
        ],
      }),
    ]
  }

  /// Bins the lights, then adds them over what the sun left in the light target.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    bind_groups: &[wgpu::BindGroup; 2],
    textures: &wgpu::BindGroup,
  ) {
    {
      let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("light binning"),
        timestamp_writes: None,
      });

      pass.set_pipeline(&self.binning);
      pass.set_bind_group(0, &bind_groups[0], &[]);
      pass.dispatch_workgroups(LIGHT_CLUSTERS.div_ceil(BINNING_WORKGROUP), 1, 1);
    }

    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("lights"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &targets.light,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, &bind_groups[1], &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    binning_layout: &wgpu::BindGroupLayout,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(wgpu::ComputePipeline, wgpu::RenderPipeline)> {
    let binning_module: wgpu::ShaderModule = create_module(device, shaders, "frame/light_binning")?;
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/lights")?;
    let binning_pipeline_layout: wgpu::PipelineLayout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("light binning"),
        bind_group_layouts: &[Some(binning_layout)],
        ..Default::default()
      });
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("lights"),
      bind_group_layouts: &[Some(view_layout), Some(layout), Some(texture_layout)],
      ..Default::default()
    });
    let binning: wgpu::ComputePipeline = create_checked(device, "light binning", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("light binning"),
        layout: Some(&binning_pipeline_layout),
        module: &binning_module,
        entry_point: Some("bin"),
        compilation_options: Default::default(),
        cache: None,
      })
    })?;
    // Each light adds to what the sun and the lights before it left.
    let additive: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };
    let pipeline: wgpu::RenderPipeline = create_checked(device, "lights", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("lights"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_fullscreen"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some("fs_lights"),
          compilation_options: Default::default(),
          targets: &[Some(wgpu::ColorTargetState {
            format: ViewTargets::LIGHT,
            blend: Some(wgpu::BlendState {
              color: additive,
              alpha: additive,
            }),
            write_mask: wgpu::ColorWrites::ALL,
          })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    })?;

    Ok((binning, pipeline))
  }
}
