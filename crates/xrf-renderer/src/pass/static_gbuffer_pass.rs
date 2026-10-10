use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

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
/// the impostors standing in for distant trees.
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
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = StaticDrawParameters::create_layout(device);
    let impostor_layout: wgpu::BindGroupLayout = StaticImpostorParameters::create_layout(device);
    let pipelines: GBufferPipelines = Self::create_pipelines(device, source, view_layout, &layout, texture_layout)?;
    let impostor_pipelines =
      Self::create_impostor_pipelines(device, source, view_layout, &impostor_layout, texture_layout)?;

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

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(device, source, &self.view_layout, &self.layout, &self.texture_layout).and_then(
        |pipelines| {
          let impostors = Self::create_impostor_pipelines(
            device,
            source,
            &self.view_layout,
            &self.impostor_layout,
            &self.texture_layout,
          )?;

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
    is_first: bool,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, prepass) in StaticBatch::list_deferred().zip(&self.pipelines.prepasses) {
      if let Some(prepass) = prepass {
        context.bind(&draws.layouts[batch.layout.get_index()]);
        context.get_pass().set_pipeline(prepass);
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }

    for (batch, pipeline) in StaticBatch::list_deferred().zip(&self.pipelines.draws) {
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
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
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
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<GBufferPipelines> {
    let module: wgpu::ShaderModule = source.get_module("static/gbuffer")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static g-buffer"),
      bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(layout)],
      ..Default::default()
    });
    let targets: [Option<wgpu::ColorTargetState>; 5] = ViewTargets::get_gbuffer_targets(true);
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create = |batches: Vec<StaticBatch>, is_pick: bool| -> XrfResult<Vec<wgpu::RenderPipeline>> {
      batches
        .into_iter()
        .map(|batch| {
          let vertex: &str = batch.layout.get_vertex_entry();
          let fragment: &str = match (batch.class, is_pick) {
            (StaticClass::CutOut, false) => "fs_cut_out",
            (StaticClass::CutOut, true) => "fs_pick_cut_out",
            (class, true) if class.is_composited() => "fs_pick_composited",
            (_, false) => "fs_opaque",
            (_, true) => "fs_pick_opaque",
          };
          // A composited surface is picked as it is drawn, pulled towards the eye, so glass laid on a wall wins. A cut-out
          // is shaded where its depth-only draw left its depth, and writes none of its own.
          let (depth_compare, bias, is_written) = match batch.class {
            class if class.is_composited() => (wgpu::CompareFunction::GreaterEqual, COMPOSITED_DEPTH_BIAS, true),
            StaticClass::CutOut if !is_pick => (wgpu::CompareFunction::Equal, wgpu::DepthBiasState::default(), false),
            _ => (wgpu::CompareFunction::Greater, wgpu::DepthBiasState::default(), true),
          };

          create_checked(device, "static g-buffer", || {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
              label: Some("static g-buffer"),
              layout: Some(&pipeline_layout),
              vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some(vertex),
                compilation_options: Default::default(),
                buffers: &[],
              },
              fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(fragment),
                compilation_options: Default::default(),
                targets: if is_pick { &pick_targets } else { &targets },
              }),
              // The packer reversed the winding with the z axis it flipped, so front faces wind counter clockwise.
              primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
              },
              depth_stencil: Some(wgpu::DepthStencilState {
                format: ViewTargets::DEPTH,
                depth_write_enabled: Some(is_written),
                depth_compare: Some(depth_compare),
                stencil: Default::default(),
                bias,
              }),
              multisample: Default::default(),
              multiview_mask: None,
              cache: source.get_driver_cache(),
            })
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
    let prepasses: Vec<Option<wgpu::RenderPipeline>> = deferred
      .iter()
      .map(|batch| {
        (batch.class == StaticClass::CutOut)
          .then(|| {
            create_checked(device, "static g-buffer depth", || {
              device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("static g-buffer depth"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                  module: &module,
                  entry_point: Some(batch.layout.get_vertex_entry()),
                  compilation_options: Default::default(),
                  buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                  module: &module,
                  entry_point: Some("fs_depth_cut_out"),
                  compilation_options: Default::default(),
                  targets: &untouched,
                }),
                primitive: wgpu::PrimitiveState {
                  front_face: wgpu::FrontFace::Ccw,
                  cull_mode: Some(wgpu::Face::Back),
                  ..Default::default()
                },
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
          })
          .transpose()
      })
      .collect::<XrfResult<_>>()?;

    Ok(GBufferPipelines {
      draws: create(deferred.clone(), false)?,
      prepasses,
      picks: create(deferred, true)?,
      composited_picks: create(Self::list_composited_picks().collect(), true)?,
    })
  }
}

/// The static G-buffer's pipelines, by batch.
struct GBufferPipelines {
  /// One a batch, by its index.
  draws: Vec<wgpu::RenderPipeline>,
  /// One a batch: a cut-out batch's depth-only draw ahead of its shading, none for an opaque one.
  prepasses: Vec<Option<wgpu::RenderPipeline>>,
  /// One a batch, writing what is drawn rather than how it looks.
  picks: Vec<wgpu::RenderPipeline>,
  /// One a composited batch, in `StaticGBufferPass::list_composited_picks` order, picking what of them shows.
  composited_picks: Vec<wgpu::RenderPipeline>,
}
