use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext, RenderPipelineDescription};

use crate::feature::effects::composited::composited_pass::CompositedPass;
use crate::feature::page::pick_glass::PickGlass;
use crate::feature::scene::statics::draw::static_draw_bindings::StaticDrawBindings;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::scene::statics::draw::static_draw_permutation::StaticDrawPermutation;
use crate::feature::scene::statics::draw::static_impostor_parameters::StaticImpostorParameters;
use crate::feature::scene::statics::draw::static_mark_draw::StaticMarkDraw;
use crate::feature::scene::statics::gbuffer::static_gbuffer_draw::StaticGbufferDraw;
use crate::frame::readback::pick_target::PickTarget;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::scene::static_scene::draw::static_batch::StaticBatch;
use crate::scene::static_scene::draw::static_class::StaticClass;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

/// Draws the static scene into a viewport's G-buffer: the cut-out batches' depth alone first, and the opaque ones' too
/// where the frame asks for a depth pass, then one indirect draw a batch, its visible clusters as instances, a batch
/// whose depth was drawn shaded only where its depth is the one drawn, then one for the impostors standing in for
/// distant trees. A frame inspecting its surfaces (a wireframe, the asset checker, alpha
/// hidden, a surface debug view) draws with pipelines keeping their code; any other with pipelines without it. For the
/// overdraw's debug view it draws every batch again, counting each surface drawn over a pixel; for the helpers, the
/// spawned models' batches again where they were drawn, marking the models a view's helpers mark.
pub struct StaticGBufferPass {
  layout: wgpu::BindGroupLayout,
  impostor_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  /// The marked models' places and colours, which a view's helpers bind (`HelpersFeature::get_marks`).
  marks_layout: wgpu::BindGroupLayout,
  /// The texel the present reads where no model is marked: nothing, in alpha too.
  unmarked: wgpu::TextureView,
  pipelines: PipelineSet<GBufferPipelines>,
  impostor_pipelines: PipelineSet<ImpostorPipelines>,
}

impl StaticGBufferPass {
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = StaticDrawParameters::create_layout(device);
    let impostor_layout: wgpu::BindGroupLayout = StaticImpostorParameters::create_layout(device);
    let marks_layout: wgpu::BindGroupLayout = Self::create_marks_layout(device);
    let pipelines: GBufferPipelines = Self::create_pipelines(
      source,
      GBufferLayouts {
        view: view_layout,
        draw: &layout,
        texture: texture_layout,
        marks: &marks_layout,
      },
    )?;
    let impostor_pipelines = Self::create_impostor_pipelines(source, view_layout, &impostor_layout, texture_layout)?;
    // A texture's texels start at nothing.
    let unmarked: wgpu::TextureView = device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("helper marks none"),
        size: wgpu::Extent3d::default(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ViewTargets::MARKS,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&wgpu::TextureViewDescriptor::default());

    Ok(Self {
      pipelines: PipelineSet::new(source.cache, pipelines),
      impostor_pipelines: PipelineSet::new(source.cache, impostor_pipelines),
      impostor_layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      marks_layout,
      unmarked,
      layout,
    })
  }

  /// What the helper marks pass binds as its fourth group: the marked models' places and colours, by place ascending.
  fn create_marks_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("helper marks"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Storage { read_only: true },
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    })
  }

  /// The layout a view's helpers bind their marked models by.
  pub fn get_marks_layout(&self) -> &wgpu::BindGroupLayout {
    &self.marks_layout
  }

  /// The texel the present reads where no model is marked.
  pub fn get_unmarked(&self) -> &wgpu::TextureView {
    &self.unmarked
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    let drawn: XrfResult = self.pipelines.refresh(source.cache, || {
      Self::create_pipelines(
        source,
        GBufferLayouts {
          view: &self.view_layout,
          draw: &self.layout,
          texture: &self.texture_layout,
          marks: &self.marks_layout,
        },
      )
    });
    let impostors: XrfResult = self.impostor_pipelines.refresh(source.cache, || {
      Self::create_impostor_pipelines(source, &self.view_layout, &self.impostor_layout, &self.texture_layout)
    });

    if let Err(error) = drawn.and(impostors) {
      log::error!("Static G-buffer rejected, drawing with the last one: {error}");
    }
  }

  /// Draws the visible clusters an argument buffer lists into the G-buffer the pass draws into, as `draw` says: the
  /// opaque surfaces' depth ahead of their shading where it prepasses, and with the frame's first draw the impostors
  /// too.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    StaticDrawBindings { view, draws, textures }: StaticDrawBindings<'_>,
    args: &wgpu::Buffer,
    draw: StaticGbufferDraw,
  ) {
    let StaticGbufferDraw {
      is_first,
      is_inspected,
      is_prepassed,
    } = draw;
    let drawing: &FrameDrawPipelines = if is_inspected {
      &self.pipelines.inspected
    } else {
      &self.pipelines.plain
    };
    let shading: &[wgpu::RenderPipeline] = if is_prepassed {
      &drawing.prepassed_draws
    } else {
      &drawing.draws
    };

    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, prepass) in StaticBatch::list_deferred().zip(&drawing.prepasses) {
      if batch.class == StaticClass::CutOut || is_prepassed {
        context.bind(&draws.layouts[batch.layout.get_index()]);
        context.get_pass().set_pipeline(prepass);
        context.get_pass().draw_indirect(args, batch.get_args_offset());
      }
    }

    for (batch, pipeline) in StaticBatch::list_deferred().zip(shading) {
      context.bind(&draws.layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);
      context.get_pass().draw_indirect(args, batch.get_args_offset());
    }

    if is_first {
      context.bind(&draws.impostors);
      context.get_pass().set_pipeline(&self.impostor_pipelines.draw);
      context
        .get_pass()
        .draw_indirect(args, StaticCullScratch::IMPOSTOR_ARGS_OFFSET);
    }
  }

  /// Draws the frame's visible clusters again into a pick's one texel, through a camera narrowed to it, in the render
  /// pass the graph opened on the pick's target: the composited ones too, glass the nearest a click meets, unless the
  /// pick passes them by.
  pub fn pick(
    &self,
    context: &mut RasterContext<'_>,
    StaticDrawBindings { view, draws, textures }: StaticDrawBindings<'_>,
    args: &[&wgpu::Buffer],
    glass: PickGlass,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    let composited: &[wgpu::RenderPipeline] = match glass {
      PickGlass::PassesBy => &[],
      PickGlass::Meets => &self.pipelines.composited_picks,
    };
    let deferred = StaticBatch::list_deferred().zip(&self.pipelines.picks);

    for (batch, pipeline) in deferred.chain(Self::list_composited_picks().zip(composited)) {
      context.bind(&draws.layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_args_offset());
      }
    }

    if let Some(args) = args.first() {
      context.bind(&draws.impostors);
      context.get_pass().set_pipeline(&self.impostor_pipelines.pick);
      context
        .get_pass()
        .draw_indirect(args, StaticCullScratch::IMPOSTOR_ARGS_OFFSET);
    }
  }

  /// Draws every batch of the clusters the argument buffers list again into the target the graph opened, with no depth
  /// test, each fragment adding one: how many of the static scene's surfaces were drawn over each pixel, hidden or not.
  pub fn record_overdraw(
    &self,
    context: &mut RasterContext<'_>,
    StaticDrawBindings { view, draws, textures }: StaticDrawBindings<'_>,
    args: &[&wgpu::Buffer],
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list().zip(&self.pipelines.overdraws) {
      context.bind(&draws.layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_args_offset());
      }
    }
  }

  /// Draws the spawned models' batches of the clusters the argument buffers list again into the marks target the graph
  /// opened, over the view's depth, each fragment where its model drew the depth kept: the colour its helper marks it
  /// in (`marks`), nothing for an unmarked one.
  pub fn mark(&self, context: &mut RasterContext<'_>, draw: StaticMarkDraw<'_>) {
    context.get_pass().set_bind_group(0, &draw.view.bind_group, &[]);
    context.get_pass().set_bind_group(1, draw.textures, &[]);
    context.get_pass().set_bind_group(3, draw.marks, &[]);

    for (batch, pipeline) in Self::list_marked().zip(&self.pipelines.marks) {
      context.bind(&draw.draws.layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in draw.args {
        context.get_pass().draw_indirect(args, batch.get_args_offset());
      }
    }
  }

  /// The batches a helper marks models in: the spawned models' deferred ones, composited models left unmarked.
  fn list_marked() -> impl Iterator<Item = StaticBatch> {
    StaticBatch::list_deferred().filter(|batch| batch.layout == StaticLayout::Model)
  }

  /// The batches a pick meets composited surfaces in: either composited class, of every layout.
  fn list_composited_picks() -> impl Iterator<Item = StaticBatch> {
    StaticBatch::list().filter(|batch| batch.class.is_composited())
  }

  /// The impostors' draw and pick: a quad either way round, so neither culls a face.
  fn create_impostor_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<ImpostorPipelines> {
    // An impostor marches no relief.
    let targets: [Option<wgpu::ColorTargetState>; 5] = ViewTargets::get_gbuffer_targets();
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create =
      |fragment: &'static str, targets: &[Option<wgpu::ColorTargetState>]| -> XrfResult<wgpu::RenderPipeline> {
        source.get_render_pipeline(
          &RenderPipelineDescription::new("static impostors", "static/impostor", "vs_impostor")
            .with_fragment(fragment)
            .with_bind_group_layouts(&[Some(view_layout), Some(texture_layout), Some(layout)])
            .with_depth_stencil(wgpu::DepthStencilState {
              format: ViewTargets::DEPTH,
              depth_write_enabled: Some(true),
              depth_compare: Some(wgpu::CompareFunction::Greater),
              stencil: Default::default(),
              bias: Default::default(),
            })
            .with_targets(targets.to_vec()),
        )
      };

    Ok(ImpostorPipelines {
      draw: create("fs_impostor", &targets)?,
      pick: create("fs_pick_impostor", &pick_targets)?,
    })
  }

  fn create_pipelines(source: PipelineSource<'_>, layouts: GBufferLayouts<'_>) -> XrfResult<GBufferPipelines> {
    let GBufferLayouts {
      view: view_layout,
      draw: layout,
      texture: texture_layout,
      marks: marks_layout,
    } = layouts;
    let targets: [Option<wgpu::ColorTargetState>; 5] = ViewTargets::get_marching_gbuffer_targets();
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create_one = |GBufferPipeline {
                        label,
                        vertex,
                        fragment,
                        permutation,
                        targets,
                        depth,
                      }: GBufferPipeline<'_>|
     -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new(label, StaticDrawPermutation::MODULE, vertex)
          .with_fragment(fragment)
          .with_permutation(permutation)
          .with_bind_group_layouts(&[Some(view_layout), Some(texture_layout), Some(layout)])
          // The packer reversed the winding with the z axis it flipped, so front faces wind counter clockwise.
          .with_primitive(wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
          })
          .with_depth_stencil(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(depth.is_written),
            depth_compare: Some(depth.compare),
            stencil: Default::default(),
            bias: depth.bias,
          })
          .with_targets(targets.to_vec()),
      )
    };
    let create = |batches: Vec<StaticBatch>,
                  GBufferVariant {
                    is_pick,
                    is_inspected,
                    is_prepassed,
                  }: GBufferVariant|
     -> XrfResult<Vec<wgpu::RenderPipeline>> {
      let permutation: StaticDrawPermutation = StaticDrawPermutation {
        is_shadow_draw: false,
        is_inspected,
      };

      batches
        .into_iter()
        .map(|batch| {
          let fragment: &str = match (batch.class, is_pick) {
            (StaticClass::CutOut, false) => "fs_cut_out",
            (StaticClass::CutOut, true) => "fs_pick_cut_out",
            (class, true) if class.is_composited() => "fs_pick_composited",
            (_, false) => "fs_opaque",
            (_, true) => "fs_pick_opaque",
          };
          // A composited surface is picked as it is drawn, pulled towards the eye, so glass laid on a wall wins. A
          // cut-out, and an opaque surface after the depth pass, is shaded where its depth-only draw left its depth,
          // and writes none of its own.
          let depth: DepthTest = match batch.class {
            class if class.is_composited() => DepthTest {
              is_written: true,
              compare: wgpu::CompareFunction::GreaterEqual,
              bias: CompositedPass::DEPTH_BIAS,
            },
            StaticClass::CutOut if !is_pick => DepthTest::KEPT,
            StaticClass::Opaque if is_prepassed => DepthTest::KEPT,
            _ => DepthTest::NEAREST,
          };

          create_one(GBufferPipeline {
            label: "static g-buffer",
            vertex: batch.layout.get_vertex_entry(),
            fragment,
            permutation: &permutation,
            targets: if is_pick { &pick_targets } else { &targets },
            depth,
          })
        })
        .collect()
    };

    let deferred: Vec<StaticBatch> = StaticBatch::list_deferred().collect();
    // The G-buffer's targets left as they are: the depth-only draw writes none of them.
    let untouched: Vec<Option<wgpu::ColorTargetState>> = targets
      .iter()
      .map(|target| {
        target.clone().map(|target| wgpu::ColorTargetState {
          write_mask: wgpu::ColorWrites::empty(),
          ..target
        })
      })
      .collect();
    let create_prepasses = |is_inspected: bool| -> XrfResult<Vec<wgpu::RenderPipeline>> {
      let permutation: StaticDrawPermutation = StaticDrawPermutation {
        is_shadow_draw: false,
        is_inspected,
      };

      deferred
        .iter()
        .map(|batch| {
          let fragment: &'static str = if batch.class == StaticClass::CutOut {
            "fs_depth_cut_out"
          } else {
            "fs_depth_opaque"
          };

          create_one(GBufferPipeline {
            label: "static g-buffer depth",
            vertex: batch.layout.get_vertex_entry(),
            fragment,
            permutation: &permutation,
            targets: &untouched,
            depth: DepthTest::NEAREST,
          })
        })
        .collect()
    };
    let frame = |is_inspected: bool| -> XrfResult<FrameDrawPipelines> {
      let draws: Vec<wgpu::RenderPipeline> = create(
        deferred.clone(),
        GBufferVariant {
          is_pick: false,
          is_inspected,
          is_prepassed: false,
        },
      )?;
      let opaque: Vec<StaticBatch> = deferred
        .iter()
        .copied()
        .filter(|it| it.class == StaticClass::Opaque)
        .collect();
      let mut prepassed = create(
        opaque,
        GBufferVariant {
          is_pick: false,
          is_inspected,
          is_prepassed: true,
        },
      )?
      .into_iter();
      // Only the opaque draws change after the depth pass; the cut-outs' are the same pipelines.
      let prepassed_draws: Vec<wgpu::RenderPipeline> = deferred
        .iter()
        .zip(&draws)
        .map(|(batch, draw)| match batch.class {
          StaticClass::Opaque => prepassed.next().unwrap_or_else(|| draw.clone()),
          _ => draw.clone(),
        })
        .collect();

      Ok(FrameDrawPipelines {
        draws,
        prepassed_draws,
        prepasses: create_prepasses(is_inspected)?,
      })
    };

    // Each fragment adds one into the light target, whatever stands in front of it.
    let counted: wgpu::ColorTargetState = wgpu::ColorTargetState {
      format: ViewTargets::LIGHT,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::One,
          dst_factor: wgpu::BlendFactor::One,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent::REPLACE,
      }),
      write_mask: wgpu::ColorWrites::ALL,
    };
    let overdraws: Vec<wgpu::RenderPipeline> = StaticBatch::list()
      .map(|batch| {
        let fragment: &'static str = if batch.class == StaticClass::CutOut {
          "fs_overdraw_cut_out"
        } else {
          "fs_overdraw"
        };

        source.get_render_pipeline(
          &RenderPipelineDescription::new(
            "static overdraw",
            StaticDrawPermutation::MODULE,
            batch.layout.get_vertex_entry(),
          )
          .with_fragment(fragment)
          .with_permutation(&StaticDrawPermutation {
            is_shadow_draw: false,
            is_inspected: false,
          })
          .with_bind_group_layouts(&[Some(view_layout), Some(texture_layout), Some(layout)])
          .with_primitive(wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
          })
          .with_target(counted.clone()),
        )
      })
      .collect::<XrfResult<_>>()?;

    // A model is marked only where its fragment is the one the frame kept.
    let marks: Vec<wgpu::RenderPipeline> = Self::list_marked()
      .map(|batch| {
        source.get_render_pipeline(
          &RenderPipelineDescription::new(
            "static helper marks",
            StaticDrawPermutation::MODULE,
            batch.layout.get_vertex_entry(),
          )
          .with_fragment("fs_helper_mark")
          .with_permutation(&StaticDrawPermutation {
            is_shadow_draw: false,
            is_inspected: false,
          })
          .with_bind_group_layouts(&[
            Some(view_layout),
            Some(texture_layout),
            Some(layout),
            Some(marks_layout),
          ])
          .with_primitive(wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
          })
          .with_depth_stencil(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Equal),
            stencil: Default::default(),
            bias: Default::default(),
          })
          .with_target(ViewTargets::MARKS.into()),
        )
      })
      .collect::<XrfResult<_>>()?;

    Ok(GBufferPipelines {
      plain: frame(false)?,
      inspected: frame(true)?,
      picks: create(deferred.clone(), GBufferVariant::PICK)?,
      composited_picks: create(Self::list_composited_picks().collect(), GBufferVariant::PICK)?,
      overdraws,
      marks,
    })
  }
}

/// Which of a batch's G-buffer pipelines is made: a pick's or a draw's, with the inspections' code or without, and an
/// opaque draw shading where the depth pass left its depth.
#[derive(Clone, Copy)]
struct GBufferVariant {
  is_pick: bool,
  is_inspected: bool,
  is_prepassed: bool,
}

impl GBufferVariant {
  /// A pick, which keeps the inspections' code and draws its own depth.
  const PICK: Self = Self {
    is_pick: true,
    is_inspected: true,
    is_prepassed: false,
  };
}

/// One of the static scene's G-buffer pipelines: its label, its shader entries and permutation, its targets and its
/// depth test.
struct GBufferPipeline<'a> {
  label: &'static str,
  vertex: &'static str,
  fragment: &'static str,
  permutation: &'a StaticDrawPermutation,
  targets: &'a [Option<wgpu::ColorTargetState>],
  depth: DepthTest,
}

/// How a G-buffer pipeline tests and writes depth.
#[derive(Clone, Copy)]
struct DepthTest {
  is_written: bool,
  compare: wgpu::CompareFunction,
  bias: wgpu::DepthBiasState,
}

impl DepthTest {
  /// The nearest surface's depth written, depth reversed.
  const NEAREST: Self = Self {
    is_written: true,
    compare: wgpu::CompareFunction::Greater,
    bias: wgpu::DepthBiasState {
      constant: 0,
      slope_scale: 0.0,
      clamp: 0.0,
    },
  };
  /// Shaded only where its depth is the one an earlier draw kept, writing none.
  const KEPT: Self = Self {
    is_written: false,
    compare: wgpu::CompareFunction::Equal,
    ..Self::NEAREST
  };
}

/// The impostors' pipelines: their draw and their pick.
struct ImpostorPipelines {
  draw: wgpu::RenderPipeline,
  pick: wgpu::RenderPipeline,
}

/// The layouts the static G-buffer's pipelines bind: the camera's, the static draw's, the textures' and, for the helper
/// marks, the marked models'.
#[derive(Clone, Copy)]
struct GBufferLayouts<'a> {
  view: &'a wgpu::BindGroupLayout,
  draw: &'a wgpu::BindGroupLayout,
  texture: &'a wgpu::BindGroupLayout,
  marks: &'a wgpu::BindGroupLayout,
}

/// The static G-buffer's pipelines, by batch.
struct GBufferPipelines {
  /// A frame's draws without the inspections' code.
  plain: FrameDrawPipelines,
  /// A frame's draws keeping it.
  inspected: FrameDrawPipelines,
  /// One a batch, writing what is drawn rather than how it looks.
  picks: Vec<wgpu::RenderPipeline>,
  /// One a composited batch, in `StaticGBufferPass::list_composited_picks` order, picking what of them shows.
  composited_picks: Vec<wgpu::RenderPipeline>,
  /// One a batch of every class, in `StaticBatch::list` order, counting what it draws for the overdraw's debug view.
  overdraws: Vec<wgpu::RenderPipeline>,
  /// One a batch in `StaticGBufferPass::list_marked` order, marking the models a view's helpers mark.
  marks: Vec<wgpu::RenderPipeline>,
}

/// A frame's draws of the G-buffer by batch: one a batch, by its index; the same with the opaque ones shaded where the
/// depth pass left their depth; and each batch's depth-only draw ahead of its shading, an opaque one's drawn only where
/// the frame asks for a depth pass.
struct FrameDrawPipelines {
  draws: Vec<wgpu::RenderPipeline>,
  prepassed_draws: Vec<wgpu::RenderPipeline>,
  prepasses: Vec<wgpu::RenderPipeline>,
}
