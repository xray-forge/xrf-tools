use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::pass::compute_grid::ComputeGrid;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a cull workgroup runs, as `shaders/static/cull.wgsl` declares them.
const WORKGROUP: u32 = 64;

/// The cull's entry points, in the order its pipelines are held.
const ENTRIES: [&str; 6] = [
  "cull_impostors",
  "cull_singles",
  "cull_rows",
  "clamp_counts",
  "cull_late",
  "clamp_late",
];

/// Decides each frame which clusters are drawn, into every batch's run of the visible list: an early phase before the
/// first draw, and a late one testing what the early phase set aside against the depth that draw left.
pub struct StaticCullPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipelines: [wgpu::ComputePipeline; 6],
  /// The singles, the rows and the clamp again, for a sun cascade's view, then for a light face's.
  shadow_pipelines: [[wgpu::ComputePipeline; 3]; 3],
  grid: ComputeGrid,
  generation: u64,
}

impl StaticCullPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = StaticCullParameters::create_layout(device);
    let (pipelines, shadow_pipelines) = Self::create_pipelines(device, shaders, view_layout, &layout)?;

    Ok(Self {
      pipelines,
      shadow_pipelines,
      grid: ComputeGrid::new(device),
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout) {
        Ok((pipelines, shadow_pipelines)) => {
          self.pipelines = pipelines;
          self.shadow_pipelines = shadow_pipelines;
        }
        Err(error) => log::error!("Static cull rejected, culling with the last one: {error}"),
      }
    }
  }

  /// Decides each impostor's level of detail, culls every cluster and row by it, then clamps each batch's count to its
  /// run and sizes the late phase.
  pub fn record_early(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    parameters: &StaticCullParameters,
    params: &StaticCullParams,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

    for (pipeline, count) in [
      (&self.pipelines[0], params.impostor_count),
      (&self.pipelines[1], params.cluster_count),
      (&self.pipelines[2], params.row_count),
      (&self.pipelines[3], StaticBatch::COUNT as u32),
    ] {
      if count > 0 {
        pass.set_pipeline(pipeline);
        self.grid.dispatch(pass, count.div_ceil(WORKGROUP));
      }
    }
  }

  /// Culls every cluster and row into a shadow's view, its own lists bound, then clamps each batch's count to its run,
  /// into a compute pass several shadows' culls may share.
  pub fn record_shadow(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    parameters: &StaticCullParameters,
    params: &StaticCullParams,
    kind: ShadowCull,
  ) {
    let pipelines: &[wgpu::ComputePipeline; 3] = &self.shadow_pipelines[kind.get_index()];

    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

    for (pipeline, count) in [
      (&pipelines[0], params.cluster_count),
      (&pipelines[1], params.row_count),
      (&pipelines[2], StaticBatch::COUNT as u32),
    ] {
      if count > 0 {
        pass.set_pipeline(pipeline);
        self.grid.dispatch(pass, count.div_ceil(WORKGROUP));
      }
    }
  }

  /// Copies how many workgroups the late phase takes, as many as the early phase set aside, where its dispatch reads
  /// them.
  pub fn record_late_dispatch(&self, encoder: &mut wgpu::CommandEncoder, late: &wgpu::Buffer, dispatch: &wgpu::Buffer) {
    encoder.copy_buffer_to_buffer(late, StaticScene::LATE_DISPATCH_OFFSET, dispatch, 0, 12);
  }

  /// Tests what the early phase set aside again, then clamps the late counts.
  pub fn record_late(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    parameters: &StaticCullParameters,
    dispatch: &wgpu::Buffer,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_pipeline(&self.pipelines[4]);
    pass.dispatch_workgroups_indirect(dispatch, 0);
    pass.set_pipeline(&self.pipelines[5]);
    pass.dispatch_workgroups((StaticBatch::COUNT as u32).div_ceil(WORKGROUP), 1, 1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<([wgpu::ComputePipeline; 6], [[wgpu::ComputePipeline; 3]; 3])> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/cull")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static cull"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });
    let create = |entry: &str, constants: &[(&str, f64)]| -> XrfResult<wgpu::ComputePipeline> {
      create_checked(device, entry, || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: Some(entry),
          layout: Some(&pipeline_layout),
          module: &module,
          entry_point: Some(entry),
          compilation_options: wgpu::PipelineCompilationOptions {
            constants,
            ..Default::default()
          },
          cache: None,
        })
      })
    };

    let [impostors, singles, rows, clamp, late, clamp_late] = ENTRIES;
    let create_shadow = |kind: ShadowCull| -> XrfResult<[wgpu::ComputePipeline; 3]> {
      let constants: &[(&str, f64)] = kind.get_constants();

      Ok([
        create(singles, constants)?,
        create(rows, constants)?,
        create(clamp, constants)?,
      ])
    };

    Ok((
      [
        create(impostors, &[])?,
        create(singles, &[])?,
        create(rows, &[])?,
        create(clamp, &[])?,
        create(late, &[])?,
        create(clamp_late, &[])?,
      ],
      [
        create_shadow(ShadowCull::Cascade)?,
        create_shadow(ShadowCull::LightFace)?,
        create_shadow(ShadowCull::Water)?,
      ],
    ))
  }
}
