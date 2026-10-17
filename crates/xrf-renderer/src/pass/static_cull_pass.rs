use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::compute_grid::ComputeGrid;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_permutation::StaticCullPermutation;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;

/// The cull's entry points, in the order its pipelines are held.
const ENTRIES: [&str; 6] = [
  "cs_cull_impostors",
  "cs_cull_singles",
  "cs_cull_rows",
  "cs_clamp_counts",
  "cs_cull_late",
  "cs_clamp_late",
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
  /// Invocations a cull workgroup runs, as `shaders/static/cull.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 64;

  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = StaticCullParameters::create_layout(device);
    let (pipelines, shadow_pipelines) = Self::create_pipelines(source, view_layout, &layout)?;

    Ok(Self {
      pipelines,
      shadow_pipelines,
      grid: ComputeGrid::new(device),
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout) {
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
    params: &StaticCullUniform,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

    for (pipeline, count) in [
      (&self.pipelines[0], params.impostor_count),
      (&self.pipelines[1], params.cluster_count),
      (&self.pipelines[2], params.row_count),
      (&self.pipelines[3], StaticBatch::LIST_COUNT as u32),
    ] {
      if count > 0 {
        pass.set_pipeline(pipeline);
        self.grid.dispatch(pass, count.div_ceil(Self::WORKGROUP));
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
    params: &StaticCullUniform,
    kind: ShadowCull,
  ) {
    let pipelines: &[wgpu::ComputePipeline; 3] = &self.shadow_pipelines[kind.get_index()];

    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

    for (pipeline, count) in [
      (&pipelines[0], params.cluster_count),
      (&pipelines[1], params.row_count),
      (&pipelines[2], StaticBatch::LIST_COUNT as u32),
    ] {
      if count > 0 {
        pass.set_pipeline(pipeline);
        self.grid.dispatch(pass, count.div_ceil(Self::WORKGROUP));
      }
    }
  }

  /// Copies how many workgroups the late phase takes, as many as the early phase set aside, where its dispatch reads
  /// them.
  pub fn record_late_dispatch(&self, encoder: &mut wgpu::CommandEncoder, late: &wgpu::Buffer, dispatch: &wgpu::Buffer) {
    encoder.copy_buffer_to_buffer(
      late,
      StaticCullScratch::LATE_DISPATCH_OFFSET,
      dispatch,
      0,
      size_of::<wgpu::util::DispatchIndirectArgs>() as u64,
    );
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
    pass.dispatch_workgroups((StaticBatch::LIST_COUNT as u32).div_ceil(Self::WORKGROUP), 1, 1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<([wgpu::ComputePipeline; 6], [[wgpu::ComputePipeline; 3]; 3])> {
    let create = |entry: &'static str, permutation: StaticCullPermutation| -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(entry, StaticCullPermutation::MODULE, entry)
          .with_permutation(&permutation)
          .with_bind_group_layouts(&[Some(view_layout), Some(layout)]),
      )
    };

    let [impostors, singles, rows, clamp, late, clamp_late] = ENTRIES;
    let create_shadow = |kind: ShadowCull| -> XrfResult<[wgpu::ComputePipeline; 3]> {
      let permutation: StaticCullPermutation = kind.get_permutation();

      Ok([
        create(singles, permutation)?,
        create(rows, permutation)?,
        create(clamp, permutation)?,
      ])
    };
    let camera: StaticCullPermutation = StaticCullPermutation::default();

    Ok((
      [
        create(impostors, camera)?,
        create(singles, camera)?,
        create(rows, camera)?,
        create(clamp, camera)?,
        create(late, camera)?,
        create(clamp_late, camera)?,
      ],
      [
        create_shadow(ShadowCull::Cascade)?,
        create_shadow(ShadowCull::LightFace)?,
        create_shadow(ShadowCull::Water)?,
      ],
    ))
  }
}
