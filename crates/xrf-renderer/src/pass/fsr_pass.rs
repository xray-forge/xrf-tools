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
use crate::scene::level::feature::fsr_graph_textures::FsrGraphTextures;
use crate::scene::level::feature::fsr_targets::FsrTargets;

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
  /// Drawn texels a side of one workgroup of the depth reconstruction.
  pub(crate) const RECONSTRUCT_WORKGROUP: u32 = 8;

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
    let uniform: UniformBinding<FsrUniform> = runtime.push_uniform(uniform);
    let textures: FsrGraphTextures = FsrGraphTextures::import(graph, bindings, fsr);

    self.add_reconstruction(graph, (targets, textures, uniform), fsr.render);
    self.add_shading_change(graph, textures, uniform);

    let accumulate: FsrAccumulateParameters = self.add_accumulation(graph, (targets, textures, uniform));

    if let Some((view, shown)) = debug {
      self.add_debug_view(graph, (view, shown), (textures, uniform), accumulate);
    }
  }

  /// The reconstruction's buffer cleared to the far plane, zero reversed, the previous depth reconstructed into it, and
  /// the inputs prepared: the dilated depth and motion, the farthest depth and its half, and the frame's luma.
  fn add_reconstruction<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (targets, textures, uniform): (ViewTargetHandles, FsrGraphTextures, UniformBinding<FsrUniform>),
    render: (u32, u32),
  ) {
    let pipelines: &'a FsrPipelines = &self.pipelines;
    let reconstructed: GraphBuffer = textures.reconstructed;

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
          render.0.div_ceil(Self::RECONSTRUCT_WORKGROUP),
          render.1.div_ceil(Self::RECONSTRUCT_WORKGROUP),
          1,
        );
      });

    Self::add_stage(
      graph,
      (
        "fsr prepare inputs",
        &[
          textures.dilated_depth,
          textures.dilated_motion,
          textures.farthest_depth,
          textures.luma,
        ],
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
        &[textures.half_farthest_depth],
        &pipelines.downsample_depth,
      ),
      FsrDownsampleParameters {
        fsr: uniform,
        source: textures.farthest_depth,
      },
    );
  }

  /// The shading change pyramid, built from the luma of this frame and the last, and the shading change read off it.
  fn add_shading_change<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    textures: FsrGraphTextures,
    uniform: UniformBinding<FsrUniform>,
  ) {
    let pipelines: &'a FsrPipelines = &self.pipelines;
    let [level0, level1, level2] = textures.pyramid;

    Self::add_stage(
      graph,
      ("fsr shading pyramid", &[level0], &pipelines.shading_pyramid),
      FsrShadingPyramidParameters {
        fsr: uniform,
        dilated_motion: textures.dilated_motion,
        luma: textures.luma,
        previous_luma: textures.previous_luma,
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
      (
        "fsr shading change",
        &[textures.shading_change],
        &pipelines.shading_change,
      ),
      FsrShadingChangeParameters {
        fsr: uniform,
        level0,
        level1,
        level2,
        linear_sampler: &self.sampler,
      },
    );
  }

  /// The reactive mask, the reactivity, the luma instability and the accumulation into this frame's history; answers
  /// the accumulation's parameters, which a debug view of its share reads.
  fn add_accumulation<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (targets, textures, uniform): (ViewTargetHandles, FsrGraphTextures, UniformBinding<FsrUniform>),
  ) -> FsrAccumulateParameters<'a> {
    let pipelines: &'a FsrPipelines = &self.pipelines;
    let sampler: &'a wgpu::Sampler = &self.sampler;

    Self::add_stage(
      graph,
      ("fsr reactive", &[textures.reactive], &pipelines.reactive),
      FsrReactiveParameters {
        fsr: uniform,
        opaque: textures.opaque,
        color: targets.scene,
      },
    );
    Self::add_stage(
      graph,
      (
        "fsr prepare reactivity",
        &[textures.masks, textures.accumulation, textures.new_locks],
        &pipelines.prepare_reactivity,
      ),
      FsrPrepareReactivityParameters {
        fsr: uniform,
        dilated_depth: textures.dilated_depth,
        dilated_motion: textures.dilated_motion,
        reconstructed: StorageArray::new(textures.reconstructed),
        reactive_mask: textures.reactive,
        shading_change: textures.shading_change,
        luma: textures.luma,
        previous_accumulation: textures.previous_accumulation,
        linear_sampler: sampler,
        material_target: targets.material,
      },
    );
    Self::add_stage(
      graph,
      (
        "fsr luma instability",
        &[textures.luma_history, textures.luma_instability],
        &pipelines.luma_instability,
      ),
      FsrLumaInstabilityParameters {
        fsr: uniform,
        dilated_motion: textures.dilated_motion,
        masks: textures.masks,
        luma: textures.luma,
        previous_luma_history: textures.previous_luma_history,
        linear_sampler: sampler,
      },
    );

    let accumulate: FsrAccumulateParameters = FsrAccumulateParameters {
      fsr: uniform,
      color: targets.scene,
      dilated_motion: textures.dilated_motion,
      masks: textures.masks,
      luma_instability: textures.luma_instability,
      farthest_depth: textures.half_farthest_depth,
      new_locks: textures.new_locks,
      history: textures.previous_history,
      linear_sampler: sampler,
    };

    Self::add_stage(
      graph,
      ("fsr accumulate", &[textures.history], &pipelines.accumulate),
      accumulate,
    );

    accumulate
  }

  /// One of FSR's debug views drawn into the target beside it; nothing for a view that is not FSR's.
  fn add_debug_view<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (view, shown): (RenderDebugView, GraphTexture),
    (textures, uniform): (FsrGraphTextures, UniformBinding<FsrUniform>),
    accumulate: FsrAccumulateParameters<'a>,
  ) {
    let debug_views: &'a FsrDebugPipelines = &self.pipelines.debug_views;
    let parameters: FsrDebugViewParameters = FsrDebugViewParameters {
      fsr: uniform,
      masks: textures.masks,
      new_locks: textures.new_locks,
      luma_instability: textures.luma_instability,
      history: textures.history,
    };
    let pipeline: &'a wgpu::RenderPipeline = match view {
      RenderDebugView::FsrMasks => &debug_views.masks,
      RenderDebugView::FsrHistoryTrust => &debug_views.history_trust,
      RenderDebugView::FsrLocks => &debug_views.locks,
      RenderDebugView::FsrLumaInstability => &debug_views.luma_instability,
      // The frame share reads what the accumulation read.
      RenderDebugView::FsrFrameShare => {
        Self::add_stage(
          graph,
          ("fsr debug view", &[shown], &debug_views.frame_share),
          accumulate,
        );

        return;
      }
      _ => return,
    };

    Self::add_stage(graph, ("fsr debug view", &[shown], pipeline), parameters);
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
