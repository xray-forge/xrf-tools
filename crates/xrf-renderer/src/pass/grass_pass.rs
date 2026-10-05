use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::compute_grid::ComputeGrid;
use crate::pass::fullscreen_pipeline::buffer_binding;
use crate::pass::grass_dispatch::GrassDispatch;
use crate::pass::grass_draws::GrassDraws;
use crate::pass::layout_entries::{storage_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// The planting's entry points, in the order a frame dispatches them.
const PLANTING_ENTRIES: [&str; 8] = [
  "clear_schedule",
  "rank",
  "select_band",
  "decompress",
  "clear_counts",
  "cull",
  "arrange",
  "scatter",
];

/// Threads a planting workgroup runs.
const WORKGROUP: u32 = 64;

/// Bytes one indexed indirect draw takes.
pub const GRASS_DRAW_BYTES: u64 = 20;

/// Plants the grass around the camera as `CDetailManager` does, on the GPU, and draws a model's tufts at a time into the
/// G-buffer.
pub struct GrassPass {
  level_layout: wgpu::BindGroupLayout,
  build_layout: wgpu::BindGroupLayout,
  draw_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  planting: Vec<wgpu::ComputePipeline>,
  draw: wgpu::RenderPipeline,
  grid: ComputeGrid,
  generation: u64,
}

impl GrassPass {
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
    let level_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("grass level"),
      entries: &[
        storage_entry(0, compute, false),
        storage_entry(1, compute, false),
        storage_entry(2, compute, false),
        storage_entry(3, compute, false),
        storage_entry(4, compute, false),
        storage_entry(5, compute, false),
        storage_entry(6, compute, true),
        storage_entry(7, compute, true),
        storage_entry(8, compute, true),
      ],
    });
    let build_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("grass build"),
      entries: &(0..8)
        .map(|binding| storage_entry(binding, compute, true))
        .chain([uniform_entry(8, compute)])
        .collect::<Vec<_>>(),
    });
    let vertex: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX;
    let draw_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("grass draw"),
      entries: &[
        storage_entry(0, vertex, false),
        storage_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT, false),
        uniform_entry(2, vertex),
      ],
    });
    let layouts = [&level_layout, &build_layout, &draw_layout, view_layout, texture_layout];

    Ok(Self {
      planting: Self::create_planting(device, shaders, [layouts[0], layouts[1]])?,
      draw: Self::create_draw(device, shaders, [layouts[3], layouts[2], layouts[4]])?,
      grid: ComputeGrid::new(device),
      generation: shaders.get_generation(),
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      level_layout,
      build_layout,
      draw_layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() == self.generation {
      return;
    }

    self.generation = shaders.get_generation();

    let planting = Self::create_planting(device, shaders, [&self.level_layout, &self.build_layout]);
    let draw = Self::create_draw(
      device,
      shaders,
      [&self.view_layout, &self.draw_layout, &self.texture_layout],
    );

    match (planting, draw) {
      (Ok(planting), Ok(draw)) => {
        self.planting = planting;
        self.draw = draw;
      }
      (Err(error), _) | (_, Err(error)) => log::error!("Grass rejected, drawing with the last: {error}"),
    }
  }

  pub fn get_level_layout(&self) -> &wgpu::BindGroupLayout {
    &self.level_layout
  }

  pub fn get_build_layout(&self) -> &wgpu::BindGroupLayout {
    &self.build_layout
  }

  /// Binds what the draws read: the sorted items, the models and the sway.
  pub fn create_draw_group(
    &self,
    device: &wgpu::Device,
    sorted: &wgpu::Buffer,
    models: &wgpu::Buffer,
    wind: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("grass draw"),
      layout: &self.draw_layout,
      entries: &[
        buffer_binding(0, sorted),
        buffer_binding(1, models),
        buffer_binding(2, wind),
      ],
    })
  }

  /// Plants the frame's grass: the ring's stale cells ranked and the nearest planted, every current cell culled, and
  /// what it keeps sorted into a draw a model.
  pub fn record_plant(
    &self,
    pass: &mut wgpu::ComputePass<'_>,
    (level, build): (&wgpu::BindGroup, &wgpu::BindGroup),
    dispatch: GrassDispatch,
  ) {
    let groups = |count: u32| count.max(1).div_ceil(WORKGROUP);
    let sizes: [u32; 8] = [
      groups(dispatch.bands),
      groups(dispatch.cells),
      1,
      groups(dispatch.cells),
      groups(dispatch.models + 1),
      groups(dispatch.cells),
      1,
      groups(dispatch.capacity),
    ];
    pass.set_bind_group(0, level, &[]);
    pass.set_bind_group(1, build, &[]);

    for (pipeline, size) in self.planting.iter().zip(sizes) {
      pass.set_pipeline(pipeline);
      self.grid.dispatch(pass, size);
    }
  }

  /// Draws every model's tufts the planting sorted into the G-buffer the pass draws into, over what the static draws
  /// left.
  pub fn record_draw(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    view: &ViewBinding,
    (draw_group, textures): (&wgpu::BindGroup, &wgpu::BindGroup),
    draws: &GrassDraws<'_>,
  ) {
    pass.set_pipeline(&self.draw);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, draw_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.set_vertex_buffer(0, draws.positions.slice(..));
    pass.set_vertex_buffer(1, draws.uvs.slice(..));
    pass.set_index_buffer(draws.indices.slice(..), wgpu::IndexFormat::Uint32);

    for model in 0..draws.models {
      pass.set_immediates(0, bytemuck::bytes_of(&model));
      pass.draw_indexed_indirect(draws.args, u64::from(model) * GRASS_DRAW_BYTES);
    }
  }

  fn create_planting(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layouts: [&wgpu::BindGroupLayout; 2],
  ) -> XrfResult<Vec<wgpu::ComputePipeline>> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "grass/planting")?;
    let layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("grass planting"),
      bind_group_layouts: &[Some(layouts[0]), Some(layouts[1])],
      ..Default::default()
    });

    PLANTING_ENTRIES
      .iter()
      .map(|entry| {
        create_checked(device, entry, || {
          device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(entry),
            layout: Some(&layout),
            module: &module,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            cache: None,
          })
        })
      })
      .collect()
  }

  fn create_draw(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layouts: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<wgpu::RenderPipeline> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "grass/grass")?;
    let layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("grass"),
      bind_group_layouts: &[Some(layouts[0]), Some(layouts[1]), Some(layouts[2])],
      immediate_size: size_of::<u32>() as u32,
    });
    let targets: [Option<wgpu::ColorTargetState>; 4] = [
      Some(ViewTargets::ALBEDO.into()),
      Some(ViewTargets::NORMAL.into()),
      Some(ViewTargets::MATERIAL.into()),
      Some(ViewTargets::MOTION.into()),
    ];

    create_checked(device, "grass", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("grass"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_grass"),
          compilation_options: Default::default(),
          buffers: &[
            Some(wgpu::VertexBufferLayout {
              array_stride: 12,
              step_mode: wgpu::VertexStepMode::Vertex,
              attributes: &wgpu::vertex_attr_array![0 => Float32x3],
            }),
            Some(wgpu::VertexBufferLayout {
              array_stride: 8,
              step_mode: wgpu::VertexStepMode::Vertex,
              attributes: &wgpu::vertex_attr_array![1 => Float32x2],
            }),
          ],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some("fs_grass"),
          compilation_options: Default::default(),
          targets: &targets,
        }),
        // `CULL_NONE`, as `CDetailManager::Render` draws every tuft.
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Greater),
          stencil: Default::default(),
          bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    })
  }
}
