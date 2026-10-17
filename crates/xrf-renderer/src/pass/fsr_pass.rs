use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputePipelineDescription, FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment,
  GraphRuntime, GraphTexture, PassParameters, StorageArray, StorageArrayMut, UniformBinding,
};

use crate::contract::render_debug_view::RenderDebugView;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::fsr_accumulate_parameters::FsrAccumulateParameters;
use crate::pass::fsr_debug_view_parameters::FsrDebugViewParameters;
use crate::pass::fsr_downsample_parameters::FsrDownsampleParameters;
use crate::pass::fsr_luma_instability_parameters::FsrLumaInstabilityParameters;
use crate::pass::fsr_prepare_inputs_parameters::FsrPrepareInputsParameters;
use crate::pass::fsr_prepare_reactivity_parameters::FsrPrepareReactivityParameters;
use crate::pass::fsr_reactive_parameters::FsrReactiveParameters;
use crate::pass::fsr_reconstruct_parameters::FsrReconstructParameters;
use crate::pass::fsr_shading_change_parameters::FsrShadingChangeParameters;
use crate::pass::fsr_shading_pyramid_parameters::FsrShadingPyramidParameters;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;
use crate::scene::level::feature::fsr_frame::FsrFrame;
use crate::scene::level::feature::fsr_targets::FsrTargets;

/// Drawn texels a side of one workgroup of the depth reconstruction.
const RECONSTRUCT_WORKGROUP: u32 = 8;

/// The layouts FSR's stages bind, in their order, each its parameters'.
struct FsrLayouts {
  reconstruct: wgpu::BindGroupLayout,
  prepare_inputs: wgpu::BindGroupLayout,
  downsample: wgpu::BindGroupLayout,
  shading_pyramid: wgpu::BindGroupLayout,
  shading_change: wgpu::BindGroupLayout,
  reactive: wgpu::BindGroupLayout,
  prepare_reactivity: wgpu::BindGroupLayout,
  luma_instability: wgpu::BindGroupLayout,
  accumulate: wgpu::BindGroupLayout,
  debug_view: wgpu::BindGroupLayout,
}

impl FsrLayouts {
  fn new(device: &wgpu::Device) -> Self {
    Self {
      reconstruct: FsrReconstructParameters::create_layout(device),
      prepare_inputs: FsrPrepareInputsParameters::create_layout(device),
      downsample: FsrDownsampleParameters::create_layout(device),
      shading_pyramid: FsrShadingPyramidParameters::create_layout(device),
      shading_change: FsrShadingChangeParameters::create_layout(device),
      reactive: FsrReactiveParameters::create_layout(device),
      prepare_reactivity: FsrPrepareReactivityParameters::create_layout(device),
      luma_instability: FsrLumaInstabilityParameters::create_layout(device),
      accumulate: FsrAccumulateParameters::create_layout(device),
      debug_view: FsrDebugViewParameters::create_layout(device),
    }
  }
}

/// The pipelines of FSR's stages; the downsample once for each format it writes.
struct FsrPipelines {
  reconstruct: wgpu::ComputePipeline,
  prepare_inputs: wgpu::RenderPipeline,
  downsample_depth: wgpu::RenderPipeline,
  downsample_pyramid: wgpu::RenderPipeline,
  shading_pyramid: wgpu::RenderPipeline,
  shading_change: wgpu::RenderPipeline,
  reactive: wgpu::RenderPipeline,
  prepare_reactivity: wgpu::RenderPipeline,
  luma_instability: wgpu::RenderPipeline,
  accumulate: wgpu::RenderPipeline,
  debug_views: FsrDebugPipelines,
}

/// The pipelines of FSR's debug views, each drawing into a target of `FsrTargets::DEBUG`.
struct FsrDebugPipelines {
  masks: wgpu::RenderPipeline,
  history_trust: wgpu::RenderPipeline,
  locks: wgpu::RenderPipeline,
  luma_instability: wgpu::RenderPipeline,
  /// The accumulation again, showing how much of each pixel this frame gave.
  frame_share: wgpu::RenderPipeline,
}

/// FSR 3.1.5's upscaler (AMD, MIT) over a viewport's frame, with an exposure of one and motion drawn at the render
/// size; each of FSR's stages a pass.
pub struct FsrPass {
  layouts: FsrLayouts,
  pipelines: FsrPipelines,
  sampler: wgpu::Sampler,
  generation: u64,
}

impl FsrPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;
    let layouts: FsrLayouts = FsrLayouts::new(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layouts)?,
      layouts,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fsr"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: source.get_generation(),
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("FSR rejected, upscaling with the last one: {error}"),
      }
    }
  }

  /// Declares FSR's stages, which leave the upscaled frame in this frame's history: the reconstruction's buffer cleared
  /// to the far plane, zero reversed; the reconstruction and the inputs; the farthest depth at half size; the shading
  /// change pyramid and the shading change; the reactive mask; the reactivity; the luma instability; and the
  /// accumulation, each full-screen stage into its targets cleared first. Where `debug` names one of FSR's debug
  /// views (`RenderDebugView::is_fsr`), it is drawn last into the target beside it.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    targets: ViewTargetHandles,
    (fsr, uniform): (&'a FsrTargets, &FsrUniform),
    debug: Option<(RenderDebugView, GraphTexture)>,
  ) {
    let pipelines: &'a FsrPipelines = &self.pipelines;
    let sampler: &'a wgpu::Sampler = &self.sampler;
    let uniform: UniformBinding<FsrUniform> = runtime.push_uniform(uniform);
    let reconstructed: GraphBuffer = bindings.import_buffer(graph, "fsr reconstructed", &fsr.reconstructed);
    let mut import = |label: &'static str, view: &'a wgpu::TextureView| bindings.import_view(graph, label, view);
    let opaque: GraphTexture = import("fsr opaque", &fsr.opaque.view);
    let reactive: GraphTexture = import("fsr reactive", &fsr.reactive.view);
    let dilated_depth: GraphTexture = import("fsr dilated depth", &fsr.dilated_depth.view);
    let dilated_motion: GraphTexture = import("fsr dilated motion", &fsr.dilated_motion.view);
    let farthest_depth: GraphTexture = import("fsr farthest depth", &fsr.farthest_depth.view);
    let half_farthest_depth: GraphTexture = import("fsr half farthest depth", &fsr.half_farthest_depth.view);
    let [level0, level1, level2] = fsr
      .shading_pyramid
      .each_ref()
      .map(|level| import("fsr shading pyramid", &level.view));
    let shading_change: GraphTexture = import("fsr shading change", &fsr.shading_change.view);
    let masks: GraphTexture = import("fsr masks", &fsr.masks.view);
    let new_locks: GraphTexture = import("fsr new locks", &fsr.new_locks.view);
    let luma_instability: GraphTexture = import("fsr luma instability", &fsr.luma_instability.view);
    // This frame's targets, written, and the last frame's, read.
    let sides: [&'a FsrFrame; 2] = fsr.frames.get_sides();
    let [luma, previous_luma] = sides.map(|frame| import("fsr luma", &frame.luma.view));
    let [accumulation, previous_accumulation] = sides.map(|frame| import("fsr accumulation", &frame.accumulation.view));
    let [luma_history, previous_luma_history] = sides.map(|frame| import("fsr luma history", &frame.luma_history.view));
    let [history, previous_history] = sides.map(|frame| import("fsr history", &frame.history.view));

    graph
      .add_encoder_pass("fsr clear")
      .buffer(reconstructed, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let reconstructed: &wgpu::Buffer = context.get_buffer(reconstructed);

        context.get_encoder().clear_buffer(reconstructed, 0, None);
      });

    let reconstruct: FsrReconstructParameters = FsrReconstructParameters {
      fsr: uniform,
      depth_target: targets.depth,
      motion_target: targets.motion,
      reconstructed: StorageArrayMut::new(reconstructed),
    };

    graph
      .add_compute_pass("fsr reconstruct")
      .parameters(&reconstruct)
      .record(move |context| {
        context.bind(&reconstruct);

        let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

        pass.set_pipeline(&pipelines.reconstruct);
        pass.dispatch_workgroups(
          fsr.render.0.div_ceil(RECONSTRUCT_WORKGROUP),
          fsr.render.1.div_ceil(RECONSTRUCT_WORKGROUP),
          1,
        );
      });

    Self::add_stage(
      graph,
      (
        "fsr prepare inputs",
        &[dilated_depth, dilated_motion, farthest_depth, luma],
        &pipelines.prepare_inputs,
      ),
      FsrPrepareInputsParameters {
        fsr: uniform,
        depth_target: targets.depth,
        motion_target: targets.motion,
        color: targets.scene,
      },
    );
    Self::add_stage(
      graph,
      (
        "fsr farthest depth",
        &[half_farthest_depth],
        &pipelines.downsample_depth,
      ),
      FsrDownsampleParameters {
        fsr: uniform,
        source: farthest_depth,
      },
    );
    Self::add_stage(
      graph,
      ("fsr shading pyramid", &[level0], &pipelines.shading_pyramid),
      FsrShadingPyramidParameters {
        fsr: uniform,
        dilated_motion,
        luma,
        previous_luma,
      },
    );

    for (source, level) in [(level0, level1), (level1, level2)] {
      Self::add_stage(
        graph,
        ("fsr shading pyramid", &[level], &pipelines.downsample_pyramid),
        FsrDownsampleParameters { fsr: uniform, source },
      );
    }

    Self::add_stage(
      graph,
      ("fsr shading change", &[shading_change], &pipelines.shading_change),
      FsrShadingChangeParameters {
        fsr: uniform,
        level0,
        level1,
        level2,
        linear_sampler: sampler,
      },
    );
    Self::add_stage(
      graph,
      ("fsr reactive", &[reactive], &pipelines.reactive),
      FsrReactiveParameters {
        fsr: uniform,
        opaque,
        color: targets.scene,
      },
    );
    Self::add_stage(
      graph,
      (
        "fsr prepare reactivity",
        &[masks, accumulation, new_locks],
        &pipelines.prepare_reactivity,
      ),
      FsrPrepareReactivityParameters {
        fsr: uniform,
        dilated_depth,
        dilated_motion,
        reconstructed: StorageArray::new(reconstructed),
        reactive_mask: reactive,
        shading_change,
        luma,
        previous_accumulation,
        linear_sampler: sampler,
        material_target: targets.material,
      },
    );
    Self::add_stage(
      graph,
      (
        "fsr luma instability",
        &[luma_history, luma_instability],
        &pipelines.luma_instability,
      ),
      FsrLumaInstabilityParameters {
        fsr: uniform,
        dilated_motion,
        masks,
        luma,
        previous_luma_history,
        linear_sampler: sampler,
      },
    );
    let accumulate: FsrAccumulateParameters = FsrAccumulateParameters {
      fsr: uniform,
      color: targets.scene,
      dilated_motion,
      masks,
      luma_instability,
      farthest_depth: half_farthest_depth,
      new_locks,
      history: previous_history,
      linear_sampler: sampler,
    };

    Self::add_stage(graph, ("fsr accumulate", &[history], &pipelines.accumulate), accumulate);

    let Some((view, shown)) = debug else {
      return;
    };
    let debug_views: &'a FsrDebugPipelines = &pipelines.debug_views;
    let parameters: FsrDebugViewParameters = FsrDebugViewParameters {
      fsr: uniform,
      masks,
      new_locks,
      luma_instability,
      history,
    };

    match view {
      RenderDebugView::FsrMasks => Self::add_stage(graph, ("fsr debug view", &[shown], &debug_views.masks), parameters),
      RenderDebugView::FsrHistoryTrust => Self::add_stage(
        graph,
        ("fsr debug view", &[shown], &debug_views.history_trust),
        parameters,
      ),
      RenderDebugView::FsrLocks => Self::add_stage(graph, ("fsr debug view", &[shown], &debug_views.locks), parameters),
      RenderDebugView::FsrLumaInstability => Self::add_stage(
        graph,
        ("fsr debug view", &[shown], &debug_views.luma_instability),
        parameters,
      ),
      RenderDebugView::FsrFrameShare => Self::add_stage(
        graph,
        ("fsr debug view", &[shown], &debug_views.frame_share),
        accumulate,
      ),
      _ => {}
    }
  }

  /// A full-screen stage drawing into its targets, cleared first, with its parameters.
  fn add_stage<'a, P: PassParameters + Send + Sync + 'a>(
    graph: &mut FrameGraph<'a>,
    (name, targets, pipeline): (&'static str, &[GraphTexture], &'a wgpu::RenderPipeline),
    parameters: P,
  ) {
    targets
      .iter()
      .fold(
        graph.add_raster_pass(name).parameters(&parameters),
        |builder, target| {
          builder.color(GraphColorAttachment::new(
            *target,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
        },
      )
      .record(move |context| {
        context.bind(&parameters);

        let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

        pass.set_pipeline(pipeline);
        pass.draw(0..3, 0..1);
      });
  }

  fn create_pipelines(source: PipelineSource<'_>, layouts: &FsrLayouts) -> XrfResult<FsrPipelines> {
    let draw = |module: &'static str,
                fragment: &'static str,
                layout: &wgpu::BindGroupLayout,
                formats: &[wgpu::TextureFormat]| {
      let targets: Vec<Option<wgpu::ColorTargetState>> = formats.iter().map(|format| Some((*format).into())).collect();

      create_fullscreen_pipeline_into(source, module, fragment, &[Some(layout)], &targets)
    };
    Ok(FsrPipelines {
      reconstruct: source.get_compute_pipeline(
        &ComputePipelineDescription::new("fsr reconstruct", "frame/fsr/reconstruct", "cs_reconstruct")
          .with_bind_group_layouts(&[Some(&layouts.reconstruct)]),
      )?,
      prepare_inputs: draw(
        "frame/fsr/prepare_inputs",
        "fs_prepare_inputs",
        &layouts.prepare_inputs,
        &[
          FsrTargets::DEPTH,
          FsrTargets::MOTION,
          FsrTargets::LUMA,
          FsrTargets::LUMA,
        ],
      )?,
      downsample_depth: draw(
        "frame/fsr/downsample",
        "fs_downsample",
        &layouts.downsample,
        &[FsrTargets::LUMA],
      )?,
      downsample_pyramid: draw(
        "frame/fsr/downsample",
        "fs_downsample",
        &layouts.downsample,
        &[FsrTargets::PYRAMID],
      )?,
      shading_pyramid: draw(
        "frame/fsr/shading_pyramid",
        "fs_shading_pyramid",
        &layouts.shading_pyramid,
        &[FsrTargets::PYRAMID],
      )?,
      shading_change: draw(
        "frame/fsr/shading_change",
        "fs_shading_change",
        &layouts.shading_change,
        &[FsrTargets::MASK],
      )?,
      reactive: draw(
        "frame/fsr/reactive",
        "fs_reactive",
        &layouts.reactive,
        &[FsrTargets::MASK],
      )?,
      prepare_reactivity: draw(
        "frame/fsr/prepare_reactivity",
        "fs_prepare_reactivity",
        &layouts.prepare_reactivity,
        &[FsrTargets::MASKS, FsrTargets::MASK, FsrTargets::MASK],
      )?,
      luma_instability: draw(
        "frame/fsr/luma_instability",
        "fs_luma_instability",
        &layouts.luma_instability,
        &[FsrTargets::LUMA_HISTORY, FsrTargets::LUMA],
      )?,
      accumulate: draw(
        "frame/fsr/accumulate",
        "fs_accumulate",
        &layouts.accumulate,
        &[FsrTargets::HISTORY],
      )?,
      debug_views: FsrDebugPipelines {
        masks: draw(
          "frame/fsr/debug_view",
          "fs_masks",
          &layouts.debug_view,
          &[FsrTargets::DEBUG],
        )?,
        history_trust: draw(
          "frame/fsr/debug_view",
          "fs_history_trust",
          &layouts.debug_view,
          &[FsrTargets::DEBUG],
        )?,
        locks: draw(
          "frame/fsr/debug_view",
          "fs_locks",
          &layouts.debug_view,
          &[FsrTargets::DEBUG],
        )?,
        luma_instability: draw(
          "frame/fsr/debug_view",
          "fs_luma_instability",
          &layouts.debug_view,
          &[FsrTargets::DEBUG],
        )?,
        frame_share: draw(
          "frame/fsr/accumulate",
          "fs_frame_share",
          &layouts.accumulate,
          &[FsrTargets::DEBUG],
        )?,
      },
    })
  }
}
