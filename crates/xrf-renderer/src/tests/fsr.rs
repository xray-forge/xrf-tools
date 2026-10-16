use glam::{Mat4, Vec2, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphCompileOptions, GraphRuntime, GraphTexture, PassParameters, PipelineCache,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_debug_view::RenderDebugView;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_target_uses::ViewTargetUses;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fsr_accumulate_parameters::FsrAccumulateParameters;
use crate::pass::fsr_debug_view_parameters::FsrDebugViewParameters;
use crate::pass::fsr_downsample_parameters::FsrDownsampleParameters;
use crate::pass::fsr_luma_instability_parameters::FsrLumaInstabilityParameters;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::fsr_prepare_inputs_parameters::FsrPrepareInputsParameters;
use crate::pass::fsr_prepare_reactivity_parameters::FsrPrepareReactivityParameters;
use crate::pass::fsr_reactive_parameters::FsrReactiveParameters;
use crate::pass::fsr_reconstruct_parameters::FsrReconstructParameters;
use crate::pass::fsr_shading_change_parameters::FsrShadingChangeParameters;
use crate::pass::fsr_shading_pyramid_parameters::FsrShadingPyramidParameters;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::scene::level::feature::fsr_targets::FsrTargets;
use crate::scene::level::feature::temporal_jitter::TemporalJitter;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;

/// Builds FSR for a drawing upscaled half again, binds it to real targets and runs a frame plain and then one with
/// each of its debug views, each reading the last one's history, without a validation error and with no stage culled.
/// Skipped, and says so, on a machine with no adapter at all.
#[test]
fn upscales_two_frames_without_a_validation_error() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let device: &wgpu::Device = &context.device;
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let cache: PipelineCache = PipelineCache::new();
  let source: PipelineSource<'_> = PipelineSource {
    cache: &cache,
    device,
    shaders: &shaders,
  };
  let pass: FsrPass = FsrPass::new(source).unwrap();
  let (render, display): ((u32, u32), (u32, u32)) = ((64, 48), (96, 72));
  let targets: ViewTargets = ViewTargets::new(device, render.0, render.1);
  let mut fsr: FsrTargets = FsrTargets::new(device, render, display, ViewTargets::SCENE);
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 4.0 / 3.0, 0.2, 1000.0);
  let mut jitter: TemporalJitter = TemporalJitter::default();
  let mut runtime: GraphRuntime = GraphRuntime::new(device, &context.queue);

  let shown: FrameTexture = FrameTexture::new(
    device,
    "fsr debug view",
    display,
    FsrTargets::DEBUG,
    wgpu::TextureUsages::empty(),
  );
  let views: [Option<RenderDebugView>; 6] = [
    None,
    Some(RenderDebugView::FsrMasks),
    Some(RenderDebugView::FsrHistoryTrust),
    Some(RenderDebugView::FsrLocks),
    Some(RenderDebugView::FsrLumaInstability),
    Some(RenderDebugView::FsrFrameShare),
  ];
  let mut previous: Vec2 = Vec2::ZERO;

  for shown_view in views {
    let offset: Vec2 = jitter.next(1.5);
    let constants: FsrUniform = FsrUniform::new(
      (render, display),
      (offset, previous),
      &view,
      (TemporalJitter::get_phases(1.5), fsr.frames.get_frame()),
    );

    previous = offset;

    {
      let mut graph: FrameGraph<'_> = FrameGraph::new();
      let mut bindings: GraphBindings<'_> = GraphBindings::new();
      let handles: ViewTargetHandles = ViewTargetHandles::import(
        &mut graph,
        &mut bindings,
        (&targets, ViewTargetUses::default(), &targets.light),
      );

      let debug: Option<(RenderDebugView, GraphTexture)> =
        shown_view.map(|view| (view, bindings.import_view(&mut graph, "fsr debug view", &shown.view)));

      pass.add_passes(
        (&mut graph, &mut bindings, &mut runtime),
        handles,
        (&fsr, &constants),
        debug,
      );

      let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

      assert!(compiled.get_culled().is_empty(), "culled {:?}", compiled.get_culled());

      let commands: Vec<wgpu::CommandBuffer> = compiled
        .execute((device, &context.queue), &mut runtime, &bindings)
        .unwrap()
        .commands;

      context.queue.submit(commands);
    }

    fsr.frames.swap();
  }

  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
  assert_eq!(fsr.frames.get_frame(), 6);
  assert_eq!(
    fsr.shading_pyramid.each_ref().map(|level| level.get_size()),
    [(32, 24), (16, 12), (8, 6)]
  );
}

/// The constants FSR reads, for a lens with reversed depth: device depth to view depth by `y / (d - x)`.
#[test]
fn reads_view_depth_back_from_reversed_device_depth() {
  let view: CameraView = CameraView::new(Vec3::ZERO, Mat4::IDENTITY, 60.0, 1.0, 0.5, 100.0);
  let constants: FsrUniform = FsrUniform::new(
    ((51, 50), (100, 100)),
    (Vec2::new(0.25, -0.25), Vec2::new(-0.5, 0.5)),
    &view,
    (32, 3),
  );
  let to_view = |depth: f32| constants.device_to_view.y / (depth - constants.device_to_view.x);

  assert!((to_view(1.0) - 0.5).abs() < 1e-3);
  assert!((to_view(0.0) - 100.0).abs() < 1e-2);
  assert_eq!(constants.jitter, Vec2::new(-0.25, 0.25));
  assert_eq!(constants.previous_jitter, Vec2::new(0.5, -0.5));
  assert_eq!(constants.downscale, Vec2::new(0.51, 0.5));
  assert_eq!(constants.shading_size, Vec2::new(25.0, 25.0));
  assert_eq!(constants.frame_index, 3.0);
}

/// FSR's stages share their modules' declarations (`common` binds the uniform, `nearest` the depth and motion), so
/// their WGSL keeps its own bindings: each stage's parameters must declare every binding as its module does.
#[test]
fn every_stage_declares_its_parameters_as_its_module_does() {
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let stages: [(&str, String); 10] = [
    ("frame/fsr/reconstruct", FsrReconstructParameters::get_wgsl_bindings()),
    (
      "frame/fsr/prepare_inputs",
      FsrPrepareInputsParameters::get_wgsl_bindings(),
    ),
    ("frame/fsr/downsample", FsrDownsampleParameters::get_wgsl_bindings()),
    (
      "frame/fsr/shading_pyramid",
      FsrShadingPyramidParameters::get_wgsl_bindings(),
    ),
    (
      "frame/fsr/shading_change",
      FsrShadingChangeParameters::get_wgsl_bindings(),
    ),
    ("frame/fsr/reactive", FsrReactiveParameters::get_wgsl_bindings()),
    (
      "frame/fsr/prepare_reactivity",
      FsrPrepareReactivityParameters::get_wgsl_bindings(),
    ),
    (
      "frame/fsr/luma_instability",
      FsrLumaInstabilityParameters::get_wgsl_bindings(),
    ),
    ("frame/fsr/accumulate", FsrAccumulateParameters::get_wgsl_bindings()),
    ("frame/fsr/debug_view", FsrDebugViewParameters::get_wgsl_bindings()),
  ];

  for (module, bindings) in stages {
    let source: String = shaders.compose(module).unwrap();

    for line in bindings.lines() {
      assert!(source.contains(line), "'{module}' does not declare '{line}'");
    }
  }
}
