use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputePipelineDescription, FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment,
  GraphRuntime, GraphTexture, PassParameters, StorageArray, StorageArrayMut, UniformBinding,
};

use crate::frame::fsr_frame::FsrFrame;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::fsr_accumulate_parameters::FsrAccumulateParameters;
use crate::pass::fsr_depth_clip_parameters::FsrDepthClipParameters;
use crate::pass::fsr_dilate_parameters::FsrDilateParameters;
use crate::pass::fsr_lock_parameters::FsrLockParameters;
use crate::pass::fsr_luma_first_parameters::FsrLumaFirstParameters;
use crate::pass::fsr_luma_shading_parameters::FsrLumaShadingParameters;
use crate::pass::fsr_reactive_parameters::FsrReactiveParameters;
use crate::pass::fsr_reconstruct_parameters::FsrReconstructParameters;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;

/// Drawn texels a side of one workgroup of the depth reconstruction.
const RECONSTRUCT_WORKGROUP: u32 = 8;

/// The layouts FSR 2's stages bind, in their order, each its parameters'.
struct FsrLayouts {
  luma_first: wgpu::BindGroupLayout,
  luma_shading: wgpu::BindGroupLayout,
  reconstruct: wgpu::BindGroupLayout,
  dilate: wgpu::BindGroupLayout,
  reactive: wgpu::BindGroupLayout,
  depth_clip: wgpu::BindGroupLayout,
  lock: wgpu::BindGroupLayout,
  accumulate: wgpu::BindGroupLayout,
}

impl FsrLayouts {
  fn new(device: &wgpu::Device) -> Self {
    Self {
      luma_first: FsrLumaFirstParameters::create_layout(device),
      luma_shading: FsrLumaShadingParameters::create_layout(device),
      reconstruct: FsrReconstructParameters::create_layout(device),
      dilate: FsrDilateParameters::create_layout(device),
      reactive: FsrReactiveParameters::create_layout(device),
      depth_clip: FsrDepthClipParameters::create_layout(device),
      lock: FsrLockParameters::create_layout(device),
      accumulate: FsrAccumulateParameters::create_layout(device),
    }
  }
}

/// The pipelines of FSR 2's stages.
struct FsrPipelines {
  luma_first: wgpu::RenderPipeline,
  luma_shading: wgpu::RenderPipeline,
  reconstruct: wgpu::ComputePipeline,
  dilate: wgpu::RenderPipeline,
  reactive: wgpu::RenderPipeline,
  depth_clip: wgpu::RenderPipeline,
  lock: wgpu::RenderPipeline,
  accumulate: wgpu::RenderPipeline,
}

/// FSR 2.2's upscaler (FidelityFX, AMD, MIT) over a viewport's frame, which is in the display's range: an exposure of
/// one and no tonemap of its own, motion drawn at the render size; FSR 2's stages, each a pass.
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
        label: Some("fsr2"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: source.get_generation(),
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("FSR 2 rejected, upscaling with the last one: {error}"),
      }
    }
  }

  /// Declares FSR 2's stages, which leave the upscaled frame in this frame's history: the reconstruction's buffer
  /// cleared (`ClearResourcesForNextFrame`, run before the reconstruction rather than after the lock: the far plane,
  /// zero reversed), the luma, the reconstruction, the dilation, the reactive mask, the depth clip, the locks and the
  /// accumulation, each full-screen stage into its targets cleared first.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    targets: ViewTargetHandles,
    (fsr, uniform): (&'a FsrTargets, &FsrUniform),
  ) {
    let pipelines: &'a FsrPipelines = &self.pipelines;
    let sampler: &'a wgpu::Sampler = &self.sampler;
    let uniform: UniformBinding<FsrUniform> = runtime.push_uniform(uniform);
    let reconstructed: GraphBuffer = bindings.import_buffer(graph, "fsr2 reconstructed", &fsr.reconstructed);
    let mut import = |label: &'static str, view: &'a wgpu::TextureView| bindings.import_view(graph, label, view);
    let luma_first: GraphTexture = import("fsr2 luma first", &fsr.luma_first.view);
    let luma_shading: GraphTexture = import("fsr2 luma shading", &fsr.luma_shading.view);
    let opaque: GraphTexture = import("fsr2 opaque", &fsr.opaque.view);
    let reactive: GraphTexture = import("fsr2 reactive", &fsr.reactive.view);
    let prepared: GraphTexture = import("fsr2 prepared", &fsr.prepared.view);
    let masks: GraphTexture = import("fsr2 masks", &fsr.masks.view);
    let locks: GraphTexture = import("fsr2 locks", &fsr.locks.view);
    let sides: [&'a FsrFrame; 2] = fsr.frames.get_sides();
    let dilated_depth: GraphTexture = import("fsr2 dilated depth", &sides[0].dilated_depth.view);
    let [dilated_motion, previous_dilated_motion] =
      sides.map(|frame| import("fsr2 dilated motion", &frame.dilated_motion.view));
    let lock_luma: GraphTexture = import("fsr2 lock luma", &sides[0].lock_luma.view);
    // This frame's histories, written, and the last frame's, read.
    let [history, previous_history] = sides.map(|frame| import("fsr2 history", &frame.history.view));
    let [lock_status, previous_lock_status] = sides.map(|frame| import("fsr2 lock status", &frame.lock_status.view));
    let [luma_history, previous_luma_history] =
      sides.map(|frame| import("fsr2 luma history", &frame.luma_history.view));

    graph
      .add_encoder_pass("fsr2 clear")
      .buffer(reconstructed, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let reconstructed: &wgpu::Buffer = context.get_buffer(reconstructed);

        context.get_encoder().clear_buffer(reconstructed, 0, None);
      });

    Self::add_stage(
      graph,
      ("fsr2 luma first", &[luma_first], &pipelines.luma_first),
      FsrLumaFirstParameters {
        fsr: uniform,
        color: targets.scene,
        linear_sampler: sampler,
      },
    );
    Self::add_stage(
      graph,
      ("fsr2 luma shading", &[luma_shading], &pipelines.luma_shading),
      FsrLumaShadingParameters {
        fsr: uniform,
        first_step: luma_first,
      },
    );

    let reconstruct: FsrReconstructParameters = FsrReconstructParameters {
      fsr: uniform,
      depth_target: targets.depth,
      motion_target: targets.motion,
      reconstructed: StorageArrayMut::new(reconstructed),
    };

    graph
      .add_compute_pass("fsr2 reconstruct")
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
        "fsr2 dilate",
        &[dilated_depth, dilated_motion, lock_luma],
        &pipelines.dilate,
      ),
      FsrDilateParameters {
        fsr: uniform,
        depth_target: targets.depth,
        motion_target: targets.motion,
        color: targets.scene,
      },
    );
    Self::add_stage(
      graph,
      ("fsr2 reactive", &[reactive], &pipelines.reactive),
      FsrReactiveParameters {
        fsr: uniform,
        opaque,
        color: targets.scene,
      },
    );
    Self::add_stage(
      graph,
      ("fsr2 depth clip", &[prepared, masks], &pipelines.depth_clip),
      FsrDepthClipParameters {
        fsr: uniform,
        color: targets.scene,
        motion_target: targets.motion,
        reconstructed: StorageArray::new(reconstructed),
        dilated_depth,
        dilated_motion,
        previous_dilated_motion,
        reactive_mask: reactive,
        linear_sampler: sampler,
      },
    );
    Self::add_stage(
      graph,
      ("fsr2 lock", &[locks], &pipelines.lock),
      FsrLockParameters {
        fsr: uniform,
        lock_luma,
      },
    );
    Self::add_stage(
      graph,
      (
        "fsr2 accumulate",
        &[history, lock_status, luma_history],
        &pipelines.accumulate,
      ),
      FsrAccumulateParameters {
        fsr: uniform,
        prepared,
        reactive_masks: masks,
        dilated_motion,
        locks,
        shading_luma: luma_shading,
        history: previous_history,
        lock_status: previous_lock_status,
        luma_history: previous_luma_history,
        linear_sampler: sampler,
      },
    );
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
      luma_first: draw(
        "frame/fsr/luma_first",
        "fs_luma_first",
        &layouts.luma_first,
        &[FsrTargets::LUMA],
      )?,
      luma_shading: draw(
        "frame/fsr/luma_shading",
        "fs_luma_shading",
        &layouts.luma_shading,
        &[FsrTargets::LUMA],
      )?,
      reconstruct: source.get_compute_pipeline(
        &ComputePipelineDescription::new("fsr2 reconstruct", "frame/fsr/reconstruct", "cs_reconstruct")
          .with_bind_group_layouts(&[Some(&layouts.reconstruct)]),
      )?,
      dilate: draw(
        "frame/fsr/dilate",
        "fs_dilate",
        &layouts.dilate,
        &[FsrTargets::DEPTH, FsrTargets::PAIR, FsrTargets::LUMA],
      )?,
      reactive: draw(
        "frame/fsr/reactive",
        "fs_reactive",
        &layouts.reactive,
        &[FsrTargets::MASK],
      )?,
      depth_clip: draw(
        "frame/fsr/depth_clip",
        "fs_depth_clip",
        &layouts.depth_clip,
        &[FsrTargets::PREPARED, FsrTargets::PAIR],
      )?,
      lock: draw("frame/fsr/lock", "fs_lock", &layouts.lock, &[FsrTargets::MASK])?,
      accumulate: draw(
        "frame/fsr/accumulate",
        "fs_accumulate",
        &layouts.accumulate,
        &[FsrTargets::HISTORY, FsrTargets::PAIR, FsrTargets::LUMA_HISTORY],
      )?,
    })
  }
}
