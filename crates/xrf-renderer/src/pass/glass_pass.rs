use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputeContext, ComputePipelineDescription, PassParameters, RasterContext, RenderPipelineDescription,
};

use crate::frame::view_targets::ViewTargets;
use crate::pass::composited_pass::CompositedPass;
use crate::pass::glass_copy_dispatch_parameters::GlassCopyDispatchParameters;
use crate::pass::glass_copy_parameters::GlassCopyParameters;
use crate::pass::glass_copy_permutation::GlassCopyPermutation;
use crate::pass::glass_parameters::GlassParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;

/// Draws a viewport's visible glass as the enhanced glass draws it, over the frame and its copy taken just before:
/// what stands behind refracted, the sky reflected by the fresnel and the sun's highlight, written whole rather than
/// blended, tested against the G-buffer's depth as the composited surfaces are, without writing it. The copy is taken
/// only where the cull lists glass to draw, so glass in the frustum that nearer surfaces hide costs no copy.
pub struct GlassPass {
  layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 4],
  /// One a layout, in `StaticLayout::ALL` order.
  pipelines: Vec<wgpu::RenderPipeline>,
  /// The copy's dispatch worked out and the copy itself: their groups, then their pipelines.
  copy_layouts: [wgpu::BindGroupLayout; 2],
  copy_pipelines: [wgpu::ComputePipeline; 2],
  /// Reads the frame's copy between its texels, clamped at its edges.
  scene_sampler: wgpu::Sampler,
  generation: u64,
}

impl GlassPass {
  /// Where its pipelines hold the sky's group.
  const SKY_GROUP: u32 = 4;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    scene_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = GlassParameters::create_layout(device);
    let copy_layouts: [wgpu::BindGroupLayout; 2] = [
      GlassCopyDispatchParameters::create_layout(device),
      GlassCopyParameters::create_layout(device),
    ];
    let layouts: [wgpu::BindGroupLayout; 4] = [
      view_layout.clone(),
      scene_layout.clone(),
      texture_layout.clone(),
      sky_layout.clone(),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layouts, &layout)?,
      copy_pipelines: Self::create_copy_pipelines(source, &copy_layouts)?,
      copy_layouts,
      layouts,
      scene_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("glass scene"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn get_scene_sampler(&self) -> &wgpu::Sampler {
    &self.scene_sampler
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Glass rejected, drawing with the last one: {error}"),
      }

      match Self::create_copy_pipelines(source, &self.copy_layouts) {
        Ok(pipelines) => self.copy_pipelines = pipelines,
        Err(error) => log::error!("Glass copy rejected, copying with the last one: {error}"),
      }
    }
  }

  /// Works out the copy's dispatch from the cull's glass lists: the whole frame where any glass draws, none where not.
  pub fn record_copy_dispatch(&self, context: &mut ComputeContext<'_>, parameters: &GlassCopyDispatchParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.copy_pipelines[0]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  /// Copies the frame the glass refracts by the dispatch worked out before it.
  pub fn record_copy(
    &self,
    context: &mut ComputeContext<'_>,
    parameters: &GlassCopyParameters,
    dispatch: &wgpu::Buffer,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.copy_pipelines[1]);
    pass.dispatch_workgroups_indirect(dispatch, 0);
  }

  /// Draws the glass each argument buffer lists, every layout's run of the glass draw list, unsorted and strictly
  /// sorted surfaces alike: each reads the frame's copy, so none needs what another drew.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (glass, sky): (&GlassParameters<'_>, &SkyParameters<'_>),
    args: &[&wgpu::Buffer],
  ) {
    context.bind(glass);
    context.bind_at(Self::SKY_GROUP, sky);
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (layout, pipeline) in StaticLayout::ALL.into_iter().zip(&self.pipelines) {
      let list: u64 = StaticBatch::get_pass_list(layout, StaticSurfacePass::Glass).into();

      context.bind(&layouts[layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, list * 16);
      }
    }
  }

  /// The copy's dispatch, reading each layout's glass list, and the copy.
  fn create_copy_pipelines(
    source: PipelineSource<'_>,
    [dispatch_layout, copy_layout]: &[wgpu::BindGroupLayout; 2],
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let list = |layout: StaticLayout| StaticBatch::get_pass_list(layout, StaticSurfacePass::Glass);
    let permutation: GlassCopyPermutation = GlassCopyPermutation {
      glass_list_baked: list(StaticLayout::Baked),
      glass_list_tree: list(StaticLayout::Tree),
      glass_list_model: list(StaticLayout::Model),
    };

    Ok([
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(
          "glass copy dispatch",
          "frame/glass_copy_dispatch",
          "cs_glass_copy_dispatch",
        )
        .with_permutation(&permutation)
        .with_bind_group_layouts(&[Some(dispatch_layout)]),
      )?,
      source.get_compute_pipeline(
        &ComputePipelineDescription::new("glass copy", "frame/glass_copy", "cs_glass_copy")
          .with_bind_group_layouts(&[Some(copy_layout)]),
      )?,
    ])
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    [view_layout, scene_layout, texture_layout, sky_layout]: &[wgpu::BindGroupLayout; 4],
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let bind_group_layouts: [Option<&wgpu::BindGroupLayout>; 5] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      Some(layout),
      Some(sky_layout),
    ];

    StaticLayout::ALL
      .into_iter()
      .map(|layout| {
        source.get_render_pipeline(
          &RenderPipelineDescription::new("glass", "static/glass", layout.get_vertex_entry())
            .with_fragment("fs_glass")
            .with_bind_group_layouts(&bind_group_layouts)
            .with_primitive(wgpu::PrimitiveState {
              front_face: wgpu::FrontFace::Ccw,
              cull_mode: Some(wgpu::Face::Back),
              ..Default::default()
            })
            .with_depth_stencil(wgpu::DepthStencilState {
              format: ViewTargets::DEPTH,
              depth_write_enabled: Some(false),
              depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
              stencil: Default::default(),
              bias: CompositedPass::DEPTH_BIAS,
            })
            .with_target(ViewTargets::SCENE.into()),
        )
      })
      .collect()
  }
}
