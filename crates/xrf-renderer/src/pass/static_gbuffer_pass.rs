use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::pick_target::PickTarget;
use crate::frame::view_targets::ViewTargets;
use crate::pass::composited_pass::COMPOSITED_DEPTH_BIAS;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_impostor_parameters::StaticImpostorParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;

/// Draws the static scene into a viewport's G-buffer: the cut-out batches' depth alone first, then one indirect draw a
/// batch, its visible clusters as instances, a cut-out batch shaded only where its depth is the one drawn, then one for
/// the impostors standing in for distant trees. A frame inspecting its surfaces (a wireframe, the asset checker, alpha
/// hidden, a surface debug view) draws with pipelines keeping their code; any other with pipelines without it.
pub struct StaticGBufferPass {
  layout: wgpu::BindGroupLayout,
  impostor_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  pipelines: GBufferPipelines,
  /// The impostors' draw, then their pick.
  impostor_pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
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
    let pipelines: GBufferPipelines = Self::create_pipelines(source, view_layout, &layout, texture_layout)?;
    let impostor_pipelines = Self::create_impostor_pipelines(source, view_layout, &impostor_layout, texture_layout)?;

    Ok(Self {
      pipelines,
      impostor_pipelines,
      impostor_layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout, &self.texture_layout).and_then(
        |pipelines| {
          let impostors =
            Self::create_impostor_pipelines(source, &self.view_layout, &self.impostor_layout, &self.texture_layout)?;

          Ok((pipelines, impostors))
        },
      ) {
        Ok((pipelines, impostor_pipelines)) => {
          self.pipelines = pipelines;
          self.impostor_pipelines = impostor_pipelines;
        }
        Err(error) => log::error!("Static G-buffer rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Draws the visible clusters an argument buffer lists into the G-buffer the pass draws into, and with the frame's
  /// first draw the impostors too: no occlusion sets one aside.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    (view, draws, textures): (&ViewBinding, &StaticDraws, &wgpu::BindGroup),
    args: &wgpu::Buffer,
    (is_first, is_inspected): (bool, bool),
  ) {
    let drawing: &FrameDrawPipelines = &self.pipelines.frames[usize::from(is_inspected)];

    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, prepass) in StaticBatch::list_deferred().zip(&drawing.prepasses) {
      if let Some(prepass) = prepass {
        context.bind(&draws.layouts[batch.layout.get_index()]);
        context.get_pass().set_pipeline(prepass);
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }

    for (batch, pipeline) in StaticBatch::list_deferred().zip(&drawing.draws) {
      context.bind(&draws.layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);
      context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
    }

    if is_first {
      context.bind(&draws.impostors);
      context.get_pass().set_pipeline(&self.impostor_pipelines[0]);
      context
        .get_pass()
        .draw_indirect(args, StaticCullScratch::IMPOSTOR_ARGS_OFFSET);
    }
  }

  /// Draws the frame's visible clusters again into a pick's one texel, through a camera narrowed to it, in the render
  /// pass the graph opened on the pick's target: the composited ones too, glass the nearest a click meets, unless the
  /// pick passes them by (`is_through`).
  pub fn pick(
    &self,
    context: &mut RasterContext<'_>,
    (view, draws, textures): (&ViewBinding, &StaticDraws, &wgpu::BindGroup),
    args: &[&wgpu::Buffer],
    is_through: bool,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    let composited: &[wgpu::RenderPipeline] = if is_through {
      &[]
    } else {
      &self.pipelines.composited_picks
    };
    let deferred = StaticBatch::list_deferred().zip(&self.pipelines.picks);

    for (batch, pipeline) in deferred.chain(Self::list_composited_picks().zip(composited)) {
      context.bind(&draws.layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pipeline);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }

    if let Some(args) = args.first() {
      context.bind(&draws.impostors);
      context.get_pass().set_pipeline(&self.impostor_pipelines[1]);
      context
        .get_pass()
        .draw_indirect(args, StaticCullScratch::IMPOSTOR_ARGS_OFFSET);
    }
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
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let device: &wgpu::Device = source.device;

    let module: wgpu::ShaderModule = source.get_module("static/impostor")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static impostors"),
      bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(layout)],
      ..Default::default()
    });
    // An impostor marches no relief.
    let targets: [Option<wgpu::ColorTargetState>; 5] = ViewTargets::get_gbuffer_targets(false);
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create = |fragment: &str, targets: &[Option<wgpu::ColorTargetState>]| -> XrfResult<wgpu::RenderPipeline> {
      create_checked(device, "static impostors", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("static impostors"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_impostor"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets,
          }),
          primitive: Default::default(),
          depth_stencil: Some(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: Default::default(),
            bias: Default::default(),
          }),
          multisample: Default::default(),
          multiview_mask: None,
          cache: source.get_driver_cache(),
        })
      })
    };

    Ok([
      create("fs_impostor", &targets)?,
      create("fs_pick_impostor", &pick_targets)?,
    ])
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<GBufferPipelines> {
    let targets: [Option<wgpu::ColorTargetState>; 5] = ViewTargets::get_gbuffer_targets(true);
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create_one = |label: &'static str,
                      (vertex, fragment, constants): (&'static str, &'static str, &[(&'static str, f64)]),
                      targets: &[Option<wgpu::ColorTargetState>],
                      (is_written, depth_compare, bias): (bool, wgpu::CompareFunction, wgpu::DepthBiasState)|
     -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new(label, "static/gbuffer", vertex)
          .with_fragment(fragment)
          .with_constants(constants)
          .with_bind_group_layouts(&[Some(view_layout), Some(texture_layout), Some(layout)])
          // The packer reversed the winding with the z axis it flipped, so front faces wind counter clockwise.
          .with_primitive(wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
          })
          .with_depth_stencil(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(is_written),
            depth_compare: Some(depth_compare),
            stencil: Default::default(),
            bias,
          })
          .with_targets(targets.to_vec()),
      )
    };
    let create =
      |batches: Vec<StaticBatch>, (is_pick, is_inspected): (bool, bool)| -> XrfResult<Vec<wgpu::RenderPipeline>> {
        let constants: [(&'static str, f64); 1] = [("IS_INSPECTED", f64::from(u8::from(is_inspected)))];

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
            // cut-out is shaded where its depth-only draw left its depth, and writes none of its own.
            let depth: (bool, wgpu::CompareFunction, wgpu::DepthBiasState) = match batch.class {
              class if class.is_composited() => (true, wgpu::CompareFunction::GreaterEqual, COMPOSITED_DEPTH_BIAS),
              StaticClass::CutOut if !is_pick => (false, wgpu::CompareFunction::Equal, wgpu::DepthBiasState::default()),
              _ => (true, wgpu::CompareFunction::Greater, wgpu::DepthBiasState::default()),
            };

            create_one(
              "static g-buffer",
              (batch.layout.get_vertex_entry(), fragment, &constants),
              if is_pick { &pick_targets } else { &targets },
              depth,
            )
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
    let create_prepasses = |is_inspected: bool| -> XrfResult<Vec<Option<wgpu::RenderPipeline>>> {
      let constants: [(&'static str, f64); 1] = [("IS_INSPECTED", f64::from(u8::from(is_inspected)))];

      deferred
        .iter()
        .map(|batch| {
          (batch.class == StaticClass::CutOut)
            .then(|| {
              create_one(
                "static g-buffer depth",
                (batch.layout.get_vertex_entry(), "fs_depth_cut_out", &constants),
                &untouched,
                (true, wgpu::CompareFunction::Greater, wgpu::DepthBiasState::default()),
              )
            })
            .transpose()
        })
        .collect::<XrfResult<_>>()
    };
    let frame = |is_inspected: bool| -> XrfResult<FrameDrawPipelines> {
      Ok(FrameDrawPipelines {
        draws: create(deferred.clone(), (false, is_inspected))?,
        prepasses: create_prepasses(is_inspected)?,
      })
    };

    Ok(GBufferPipelines {
      frames: [frame(false)?, frame(true)?],
      picks: create(deferred.clone(), (true, true))?,
      composited_picks: create(Self::list_composited_picks().collect(), (true, true))?,
    })
  }
}

/// The static G-buffer's pipelines, by batch: a frame's draws without the inspections' code, then with it.
struct GBufferPipelines {
  frames: [FrameDrawPipelines; 2],
  /// One a batch, writing what is drawn rather than how it looks.
  picks: Vec<wgpu::RenderPipeline>,
  /// One a composited batch, in `StaticGBufferPass::list_composited_picks` order, picking what of them shows.
  composited_picks: Vec<wgpu::RenderPipeline>,
}

/// A frame's draws of the G-buffer by batch: one a batch, by its index, and a cut-out batch's depth-only draw ahead of
/// its shading, none for an opaque one.
struct FrameDrawPipelines {
  draws: Vec<wgpu::RenderPipeline>,
  prepasses: Vec<Option<wgpu::RenderPipeline>>,
}
