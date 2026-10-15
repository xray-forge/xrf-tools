use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::composited_pipelines::CompositedPipelines;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_geometry::StaticGeometry;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;

/// The fragment entry compositing surfaces over the lit frame.
const COMPOSITED_FRAGMENT: &str = "fs_composited";

/// `out = colour + behind × second colour`, by dual-source blending: over by alpha, added and multiplied all in one.
const COMPOSITED_BLEND: wgpu::BlendState = wgpu::BlendState {
  color: wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::Src1,
    operation: wgpu::BlendOperation::Add,
  },
  alpha: wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::One,
    dst_factor: wgpu::BlendFactor::Src1Alpha,
    operation: wgpu::BlendOperation::Add,
  },
};

/// Draws a viewport's visible blended, added and multiplied static surfaces over its lit scene, its wall marks into its
/// G-buffer's albedo before any light, its distorting surfaces' special pass into the distortion target, and its
/// selected ones' mark into the material target, each tested against the G-buffer's depth without writing it.
pub struct CompositedPass {
  layout: wgpu::BindGroupLayout,
  layouts: [wgpu::BindGroupLayout; 4],
  pipelines: CompositedPipelines,
  generation: u64,
}

impl CompositedPass {
  /// Reversed depth steps a composited surface is pulled towards the eye by, so a surface laid on a wall never loses
  /// to it, and the same scaled by its slope for one seen at a grazing angle.
  pub const DEPTH_BIAS: wgpu::DepthBiasState = wgpu::DepthBiasState {
    constant: 256,
    slope_scale: 1.0,
    clamp: 0.0,
  };

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

    // A forward-drawn model is lit, and its sun read, per vertex.
    let layout: wgpu::BindGroupLayout = CompositedParameters::create_layout(device);
    let layouts: [wgpu::BindGroupLayout; 4] = [
      view_layout.clone(),
      scene_layout.clone(),
      texture_layout.clone(),
      sky_layout.clone(),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layouts, &layout)?,
      layouts,
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Composited surfaces rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Draws the unsorted composited surfaces each argument buffer lists, in cluster order.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (composited, sky): (&CompositedParameters<'_>, &SkyParameters<'_>),
    args: &[&wgpu::Buffer],
  ) {
    Self::bind_common(context, (view, textures), (composited, sky));

    for (batch, pipeline) in StaticBatch::list_composited().zip(&self.pipelines.composited) {
      // The models' are drawn back to front from the view's sorted list rather than as the cull listed them.
      if batch.layout == StaticLayout::Model {
        continue;
      }

      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  /// Draws the view's sorted composited clusters back to front: each run of them alike in layout, as `(layout, first
  /// entry, count)`, by its layout's program over the sorted list `sorted` reads.
  pub fn record_sorted(
    &self,
    context: &mut RasterContext<'_>,
    (view, sorted, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (composited, sky): (&CompositedParameters<'_>, &SkyParameters<'_>),
    runs: &[(StaticLayout, u32, u32)],
  ) {
    Self::bind_common(context, (view, textures), (composited, sky));

    for (layout, first, count) in runs {
      // The composited batches run in layout order, one a layout, so a layout's program is its batch's.
      context.bind(&sorted[layout.get_index()]);
      context.get_pass().set_pipeline(&self.pipelines.composited[layout.get_index()]);
      context
        .get_pass()
        .draw(0..StaticGeometry::CLUSTER_VERTICES, *first..first + count);
    }
  }

  /// Binds what every composited draw reads: the view, the bindless textures, the lighting and the sky, whose group
  /// sits above the composited surfaces' own, past the textures and the draws.
  fn bind_common(
    context: &mut RasterContext<'_>,
    (view, textures): (&ViewBinding, &wgpu::BindGroup),
    (composited, sky): (&CompositedParameters<'_>, &SkyParameters<'_>),
  ) {
    context.bind(composited);
    context.bind_at(Self::SKY_GROUP, sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);
  }

  /// Draws the distorting surfaces each argument buffer lists, every layout's run of the distortion draw list, into the
  /// distortion target, as `model_distort4glass` draws their special pass: unsorted and strictly sorted alike, in
  /// cluster order, which blending offsets makes no matter.
  pub fn record_distortion(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    (composited, sky): (&CompositedParameters<'_>, &SkyParameters<'_>),
    args: &[&wgpu::Buffer],
  ) {
    Self::bind_common(context, (view, textures), (composited, sky));

    for (layout, pipeline) in StaticLayout::ALL.into_iter().zip(&self.pipelines.distortion) {
      let list: u64 = StaticBatch::get_pass_list(layout, StaticSurfacePass::Distortion).into();

      context.bind(&layouts[layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, list * 16);
      }
    }
  }

  /// Marks the selected surfaces among the composited ones each argument buffer lists in the material target's alpha,
  /// which the present outlines the selection by.
  pub fn record_selection(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &[&wgpu::Buffer],
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in Self::list_selectable().zip(&self.pipelines.selection) {
      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  /// The batches a selected composited surface may stand in: either composited class, of every layout.
  fn list_selectable() -> impl Iterator<Item = StaticBatch> {
    StaticBatch::list().filter(|batch| batch.class.is_composited())
  }

  /// Lays the wall marks each argument buffer lists into the G-buffer's albedo, before any light reads it.
  pub fn record_wallmarks(
    &self,
    context: &mut RasterContext<'_>,
    (view, layouts, textures): (
      &ViewBinding,
      &[StaticDrawParameters; StaticLayout::COUNT],
      &wgpu::BindGroup,
    ),
    args: &[&wgpu::Buffer],
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list_wallmarks().zip(&self.pipelines.wallmarks) {
      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    [view_layout, scene_layout, texture_layout, sky_layout]: &[wgpu::BindGroupLayout; 4],
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<CompositedPipelines> {
    let composited_layout: [Option<&wgpu::BindGroupLayout>; 5] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      Some(layout),
      Some(sky_layout),
    ];
    let wallmark_layout: [Option<&wgpu::BindGroupLayout>; 3] =
      [Some(view_layout), Some(texture_layout), Some(scene_layout)];
    let composited = Self::create_layout_pipelines(
      source,
      (&composited_layout, COMPOSITED_FRAGMENT),
      (ViewTargets::SCENE, Some(COMPOSITED_BLEND), wgpu::ColorWrites::ALL),
      StaticBatch::list_composited().map(|batch| batch.layout),
    )?;
    let wallmarks = Self::create_layout_pipelines(
      source,
      (&wallmark_layout, "fs_wallmark"),
      (ViewTargets::ALBEDO, Some(COMPOSITED_BLEND), wgpu::ColorWrites::ALL),
      StaticBatch::list_wallmarks().map(|batch| batch.layout),
    )?;
    // Blended over what the water and the particles wrote, as `distort(true)` passes blend by their alpha.
    let distortion = Self::create_layout_pipelines(
      source,
      (&composited_layout, "fs_composited_distort"),
      (
        ViewTargets::DISTORTION,
        Some(wgpu::BlendState::ALPHA_BLENDING),
        wgpu::ColorWrites::ALL,
      ),
      StaticLayout::ALL.into_iter(),
    )?;
    // The mark alone, in the alpha, over whatever the surface behind it marked.
    let selection = Self::create_layout_pipelines(
      source,
      (&wallmark_layout, "fs_composited_selected"),
      (ViewTargets::MATERIAL, None, wgpu::ColorWrites::ALPHA),
      Self::list_selectable().map(|batch| batch.layout),
    )?;

    Ok(CompositedPipelines {
      composited,
      wallmarks,
      distortion,
      selection,
    })
  }

  fn create_layout_pipelines(
    source: PipelineSource<'_>,
    (bind_group_layouts, fragment): (&[Option<&wgpu::BindGroupLayout>], &'static str),
    (format, blend, write_mask): (wgpu::TextureFormat, Option<wgpu::BlendState>, wgpu::ColorWrites),
    layouts: impl Iterator<Item = StaticLayout>,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    layouts
      .map(|layout| {
        // A model and a vertex-lit level surface composited over the frame are lit by their vertices, as the engine's
        // forward passes light them.
        let vertex: &'static str = match layout {
          StaticLayout::Model if fragment == COMPOSITED_FRAGMENT => "vs_model_lit",
          StaticLayout::Baked if fragment == COMPOSITED_FRAGMENT => "vs_baked_lit",
          layout => layout.get_vertex_entry(),
        };

        source.get_render_pipeline(
          &RenderPipelineDescription::new(fragment, "static/composited", vertex)
            .with_fragment(fragment)
            .with_bind_group_layouts(bind_group_layouts)
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
              bias: Self::DEPTH_BIAS,
            })
            .with_target(wgpu::ColorTargetState {
              format,
              blend,
              write_mask,
            }),
        )
      })
      .collect()
  }
}
