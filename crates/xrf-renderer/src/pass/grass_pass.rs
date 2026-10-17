use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputeContext, ComputePipelineDescription, PassParameters, RasterContext, RenderPipelineDescription, StorageField,
  VertexLayout,
};

use crate::frame::view_targets::ViewTargets;
use crate::pass::compute_grid::ComputeGrid;
use crate::pass::grass_build_parameters::GrassBuildParameters;
use crate::pass::grass_card_shadow_parameters::GrassCardShadowParameters;
use crate::pass::grass_cards_parameters::GrassCardsParameters;
use crate::pass::grass_compute_pipelines::GrassComputePipelines;
use crate::pass::grass_dispatch::GrassDispatch;
use crate::pass::grass_draw_parameters::GrassDrawParameters;
use crate::pass::grass_draws::GrassDraws;
use crate::pass::grass_level_parameters::GrassLevelParameters;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::grass_sort_parameters::GrassSortParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::sun_shadow_maps::SunShadowMaps;

/// The planting's entry points, in the order a frame dispatches them: the ring's stale cells ranked and the nearest
/// planted, before the frame's depth; then, against that depth, every current cell culled and what it keeps sorted.
const PLANTING_ENTRIES: [&str; 4] = ["cs_clear_schedule", "cs_rank", "cs_select_band", "cs_decompress"];
const CULLING_ENTRIES: [&str; 2] = ["cs_clear_counts", "cs_cull"];
/// The cards' entry points: every card cleared, the near tufts' stood, then the cascades' draw of them sized.
const CARDS_ENTRIES: [&str; 3] = ["cs_clear_cards", "cs_grass_cards", "cs_card_draw"];

/// Threads a planting workgroup runs.
const WORKGROUP: u32 = 64;

/// Items the lists hold past which the sort is dispatched over the items kept, read off the GPU, rather than over every
/// item the lists hold. The indirect dispatch costs ~0.02 ms on D3D12, which patches its arguments in a pass of its
/// own; dispatching over the lists costs ~0.035 ms a million items (RTX 5080), so the indirect one wins only past ~1M.
const SORT_INDIRECT_ABOVE: u32 = 1 << 21;

/// Plants the grass around the camera as `CDetailManager` does, on the GPU, and draws a model's tufts at a time into
/// the G-buffer. Its draws bind the texture cache's group at 1, as every static draw does, and their own at 2.
pub struct GrassPass {
  level_layout: wgpu::BindGroupLayout,
  build_layout: wgpu::BindGroupLayout,
  draw_layout: wgpu::BindGroupLayout,
  occlusion_layout: wgpu::BindGroupLayout,
  /// The sort's indirect dispatch, which `arrange` alone writes.
  sort_layout: wgpu::BindGroupLayout,
  /// The models' arenas, the wind and the cards, which standing them reads and writes.
  cards_layout: wgpu::BindGroupLayout,
  /// The cards' corners and triangles and the models, which their draw into a sun cascade reads.
  card_shadow_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  planting: Vec<wgpu::ComputePipeline>,
  /// Clearing the counts and culling, then arranging the draws and sorting the items into them.
  culling: Vec<wgpu::ComputePipeline>,
  arrange: wgpu::ComputePipeline,
  sort: wgpu::ComputePipeline,
  cards: Vec<wgpu::ComputePipeline>,
  draw: wgpu::RenderPipeline,
  card_shadow: wgpu::RenderPipeline,
  grid: ComputeGrid,
  generation: u64,
}

impl GrassPass {
  /// Bytes one indexed indirect draw takes.
  pub const DRAW_BYTES: u64 = 20;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let level_layout: wgpu::BindGroupLayout = GrassLevelParameters::create_layout(device);
    let build_layout: wgpu::BindGroupLayout = GrassBuildParameters::create_layout(device);
    let draw_layout: wgpu::BindGroupLayout = GrassDrawParameters::create_layout(device);
    let occlusion_layout: wgpu::BindGroupLayout = GrassOcclusionParameters::create_layout(device);
    let sort_layout: wgpu::BindGroupLayout = GrassSortParameters::create_layout(device);
    let cards_layout: wgpu::BindGroupLayout = GrassCardsParameters::create_layout(device);
    let card_shadow_layout: wgpu::BindGroupLayout = GrassCardShadowParameters::create_layout(device);
    let GrassComputePipelines {
      planting,
      culling,
      arrange,
      sort,
      cards,
    } = Self::create_planting(
      source,
      [
        &level_layout,
        &build_layout,
        &occlusion_layout,
        &sort_layout,
        &cards_layout,
      ],
    )?;

    Ok(Self {
      planting,
      culling,
      arrange,
      sort,
      cards,
      draw: Self::create_draw(source, [view_layout, texture_layout, &draw_layout])?,
      card_shadow: Self::create_card_shadow(source, [view_layout, texture_layout, &card_shadow_layout])?,
      grid: ComputeGrid::new(device),
      generation: source.get_generation(),
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      level_layout,
      build_layout,
      draw_layout,
      occlusion_layout,
      sort_layout,
      cards_layout,
      card_shadow_layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() == self.generation {
      return;
    }

    self.generation = source.get_generation();

    let planting = Self::create_planting(
      source,
      [
        &self.level_layout,
        &self.build_layout,
        &self.occlusion_layout,
        &self.sort_layout,
        &self.cards_layout,
      ],
    );
    let draw = Self::create_draw(source, [&self.view_layout, &self.texture_layout, &self.draw_layout]);
    let card_shadow = Self::create_card_shadow(
      source,
      [&self.view_layout, &self.texture_layout, &self.card_shadow_layout],
    );

    match (planting, draw.and_then(|draw| Ok((draw, card_shadow?)))) {
      (Ok(pipelines), Ok((draw, card_shadow))) => {
        self.planting = pipelines.planting;
        self.culling = pipelines.culling;
        self.arrange = pipelines.arrange;
        self.sort = pipelines.sort;
        self.cards = pipelines.cards;
        self.draw = draw;
        self.card_shadow = card_shadow;
      }
      (Err(error), _) | (_, Err(error)) => log::error!("Grass rejected, drawing with the last: {error}"),
    }
  }

  /// Draws the cards that stand into a sun cascade's map the pass draws into, seen by its `view`, as many corners as
  /// `draw` holds.
  pub fn record_card_shadow(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    (parameters, textures): (&GrassCardShadowParameters, &wgpu::BindGroup),
    draw: &wgpu::Buffer,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.card_shadow);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);
    pass.draw_indirect(draw, 0);
  }

  /// Stands the cards: their count reset and `triangles` of them cleared, each of the `tufts` invocations' tuft stood
  /// where it is near the eye, then the cascades' draw of them sized.
  pub fn record_cards(
    &self,
    context: &mut ComputeContext<'_>,
    (level, build, cards): (&GrassLevelParameters, &GrassBuildParameters, &GrassCardsParameters),
    (triangles, tufts): (u32, u32),
  ) {
    let groups = |count: u32| count.max(1).div_ceil(WORKGROUP);

    context.bind(level);
    context.bind(build);
    context.bind(cards);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    for (pipeline, size) in self.cards.iter().zip([groups(triangles), groups(tufts), 1]) {
      pass.set_pipeline(pipeline);
      self.grid.dispatch(pass, size);
    }
  }

  /// Plants the frame's grass: the ring's stale cells ranked and the nearest planted.
  pub fn record_plant(
    &self,
    context: &mut ComputeContext<'_>,
    (level, build): (&GrassLevelParameters, &GrassBuildParameters),
    dispatch: GrassDispatch,
  ) {
    let groups = |count: u32| count.max(1).div_ceil(WORKGROUP);
    let sizes: [u32; 4] = [
      groups(dispatch.bands),
      groups(dispatch.cells),
      1,
      groups(dispatch.cells),
    ];

    context.bind(level);
    context.bind(build);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    for (pipeline, size) in self.planting.iter().zip(sizes) {
      pass.set_pipeline(pipeline);
      self.grid.dispatch(pass, size);
    }
  }

  /// Culls every current cell of the ring by the view and the frame's depth `occlusion`, and sorts what it keeps into a
  /// draw a model.
  pub fn record_cull(
    &self,
    context: &mut ComputeContext<'_>,
    (level, build, sort): (&GrassLevelParameters, &GrassBuildParameters, &GrassSortParameters),
    occlusion: &GrassOcclusionParameters,
    dispatch: GrassDispatch,
  ) {
    let groups = |count: u32| count.max(1).div_ceil(WORKGROUP);
    let sizes: [u32; 2] = [groups(dispatch.models + 1), groups(dispatch.cells)];

    context.bind(level);
    context.bind(build);
    context.bind(occlusion);

    for (pipeline, size) in self.culling.iter().zip(sizes) {
      let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

      pass.set_pipeline(pipeline);
      self.grid.dispatch(pass, size);
    }

    context.get_pass().set_pipeline(&self.arrange);
    context.bind(sort);
    context.get_pass().dispatch_workgroups(1, 1, 1);
    context.get_pass().set_pipeline(&self.sort);

    // Wide lists sort over the items kept, at a wide radius a hundredth of what they hold; narrow ones over all of
    // them.
    if dispatch.capacity > SORT_INDIRECT_ABOVE {
      let sort_dispatch: &wgpu::Buffer = context.get_buffer(sort.sort_dispatch.get_buffer());

      context.get_pass().dispatch_workgroups_indirect(sort_dispatch, 0);
    } else {
      self.grid.dispatch(context.get_pass(), groups(dispatch.capacity));
    }
  }

  /// Draws every model's tufts the planting sorted into the G-buffer the pass draws into, over what the static draws
  /// left.
  pub fn record_draw(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    (parameters, textures): (&GrassDrawParameters, &wgpu::BindGroup),
    draws: &GrassDraws<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.draw);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);
    pass.set_vertex_buffer(0, draws.positions.slice(..));
    pass.set_vertex_buffer(1, draws.uvs.slice(..));
    pass.set_index_buffer(draws.indices.slice(..), wgpu::IndexFormat::Uint32);

    for model in 0..draws.models {
      pass.set_immediates(0, bytemuck::bytes_of(&model));
      pass.draw_indexed_indirect(draws.args, u64::from(model) * Self::DRAW_BYTES);
    }
  }

  fn create_planting(
    source: PipelineSource<'_>,
    [level_layout, build_layout, occlusion_layout, sort_layout, cards_layout]: [&wgpu::BindGroupLayout; 5],
  ) -> XrfResult<GrassComputePipelines> {
    let planting: [Option<&wgpu::BindGroupLayout>; 2] = [Some(level_layout), Some(build_layout)];
    let culling: [Option<&wgpu::BindGroupLayout>; 3] = [Some(level_layout), Some(build_layout), Some(occlusion_layout)];
    let create_in = |module: &'static str, entry: &'static str, layouts: &[Option<&wgpu::BindGroupLayout>]| {
      source
        .get_compute_pipeline(&ComputePipelineDescription::new(entry, module, entry).with_bind_group_layouts(layouts))
    };
    let create =
      |entry: &'static str, layouts: &[Option<&wgpu::BindGroupLayout>]| create_in("grass/planting", entry, layouts);
    let create_all =
      |entries: &[&'static str], layouts: &[Option<&wgpu::BindGroupLayout>]| -> XrfResult<Vec<wgpu::ComputePipeline>> {
        entries.iter().map(|entry| create(entry, layouts)).collect()
      };

    Ok(GrassComputePipelines {
      planting: create_all(&PLANTING_ENTRIES, &planting)?,
      culling: create_all(&CULLING_ENTRIES, &culling)?,
      arrange: create(
        "cs_arrange",
        &[Some(level_layout), Some(build_layout), None, Some(sort_layout)],
      )?,
      sort: create("cs_scatter", &culling)?,
      cards: CARDS_ENTRIES
        .iter()
        .map(|entry| {
          create_in(
            "grass/cards",
            entry,
            &[Some(level_layout), Some(build_layout), Some(cards_layout)],
          )
        })
        .collect::<XrfResult<_>>()?,
    })
  }

  /// The cards' depth into a sun cascade, cut out by their models' textures from both sides, as the shadows' casters
  /// draw.
  fn create_card_shadow(
    source: PipelineSource<'_>,
    layouts: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<wgpu::RenderPipeline> {
    source.get_render_pipeline(
      &RenderPipelineDescription::new("grass card shadow", "grass/card_shadow", "vs_card_shadow")
        .with_fragment("fs_card_shadow")
        .with_bind_group_layouts(&layouts.map(Some))
        .with_depth_stencil(wgpu::DepthStencilState {
          format: SunShadowMaps::FORMAT,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Greater),
          stencil: Default::default(),
          bias: Default::default(),
        }),
    )
  }

  fn create_draw(source: PipelineSource<'_>, layouts: [&wgpu::BindGroupLayout; 3]) -> XrfResult<wgpu::RenderPipeline> {
    // A tuft marches no relief: what the ground under it wrote stands, which the wet pass, wetting terrain alone, reads
    // nowhere grass is drawn.
    let targets: [Option<wgpu::ColorTargetState>; 5] = ViewTargets::get_gbuffer_targets(false);

    // `CULL_NONE`, as `CDetailManager::Render` draws every tuft.
    source.get_render_pipeline(
      &RenderPipelineDescription::new("grass", "grass/grass", "vs_grass")
        .with_fragment("fs_grass")
        .with_bind_group_layouts(&layouts.map(Some))
        .with_immediate_size(size_of::<u32>() as u32)
        .with_vertex_layout(VertexLayout::new(
          12,
          wgpu::VertexStepMode::Vertex,
          &wgpu::vertex_attr_array![0 => Float32x3],
        ))
        .with_vertex_layout(VertexLayout::new(
          8,
          wgpu::VertexStepMode::Vertex,
          &wgpu::vertex_attr_array![1 => Float32x2],
        ))
        .with_depth_stencil(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Greater),
          stencil: Default::default(),
          bias: Default::default(),
        })
        .with_targets(targets.to_vec()),
    )
  }
}
