use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use wgpu::util::StagingBelt;
use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::{GraphRuntime, PipelineCache};

use crate::context::gpu_context::GpuContext;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::timing::frame_span_timer::FrameSpanTimer;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu::gpu_sources::GpuSources;
use crate::thread::gpu::gpu_state::GpuState;
use crate::thread::gpu::kept_pipelines::KeptPipelines;
use crate::thread::gpu::running_gpu::RunningGpu;
use crate::thread::gpu::window_surfaces::WindowSurfaces;
use crate::thread::queue::render_workers::RenderWorkers;

/// The GPU while its passes are built on a thread of their own, about three seconds: its device, and what a window's
/// layer (a native host's own interface, such as a game's loading view) is drawn with meanwhile, so the layer shows
/// from the device's start. Everything else a frame draws waits for the passes.
///
/// What it made goes on into the running GPU on the same device: the windows' surfaces, the pipelines it made and the
/// frames' runtime.
pub(crate) struct StartingGpu {
  /// Another handle on the device the passes are built on.
  pub context: GpuContext,
  /// The pipeline cache the passes are made through too, which the layer's pipelines go into.
  pub pipelines: Arc<PipelineCache>,
  /// The swapchains of the windows a layer was drawn into, and the layer pass.
  pub surfaces: WindowSurfaces,
  /// What the frames keep between them, the timer the viewports' reports read among it.
  pub runtime: GraphRuntime,
  /// The staging belt the layers' writes go through, kept between frames; out while a frame's encoder holds it.
  pub belt: Option<StagingBelt>,
  /// The thread building the passes, until they are taken.
  build: JoinHandle<XrfResult<GpuState>>,
}

impl StartingGpu {
  /// Starts building the passes on `context`'s device on a thread of their own, from the thread's own copies of what
  /// they are built from.
  ///
  /// # Errors
  ///
  /// Returns an error where the thread does not start.
  pub(crate) fn begin(
    context: GpuContext,
    shaders: &ShaderLibrary,
    workers: &RenderWorkers,
    bundle: &Arc<dyn RenderBundle>,
    store: Option<&Arc<dyn RenderCacheStore>>,
  ) -> XrfResult<Self> {
    let kept: KeptPipelines = KeptPipelines::open(&context, store.map(Arc::as_ref));
    let pipelines: Arc<PipelineCache> = Arc::clone(&kept.cache);
    let surfaces: WindowSurfaces = WindowSurfaces::new(PipelineSource {
      cache: &pipelines,
      device: &context.device,
      shaders,
    });
    let runtime: GraphRuntime = GraphRuntime::new(&context.device, &context.queue);
    let built: GpuContext = context.clone();
    let shaders: ShaderLibrary = shaders.clone();
    let workers: RenderWorkers = workers.clone();
    let bundle: Arc<dyn RenderBundle> = Arc::clone(bundle);
    let store: Option<Arc<dyn RenderCacheStore>> = store.cloned();
    let build: JoinHandle<XrfResult<GpuState>> = std::thread::Builder::new()
      .name("xrf-render-passes".into())
      .spawn(move || {
        let building: Instant = Instant::now();
        let state: XrfResult<GpuState> = GpuState::new(
          built,
          kept,
          GpuSources {
            shaders: &shaders,
            workers: &workers,
            bundle: bundle.as_ref(),
            store: store.as_deref(),
          },
        );

        log::info!("Renderer's passes built in {} ms", building.elapsed().as_millis());

        state
      })
      .map_err(|error| XrfError::new_unexpected_error(format!("The renderer's passes cannot be built: {error}")))?;

    Ok(Self {
      context,
      pipelines,
      surfaces,
      runtime,
      belt: None,
      build,
    })
  }

  /// Whether the passes are built, or failed to be, so `finish` answers without waiting.
  pub(crate) fn is_built(&self) -> bool {
    self.build.is_finished()
  }

  /// Opens a frame's encoder over the belt kept from the last frame, or a new one.
  pub(crate) fn begin_encoder(&mut self) -> FrameEncoder {
    let device: &wgpu::Device = &self.context.device;
    let belt: StagingBelt = self.belt.take().unwrap_or_else(|| FrameEncoder::create_belt(device));

    FrameEncoder::new(device, belt)
  }

  /// The running GPU once its passes are built, waiting for them where they are not: the windows' surfaces, the frames'
  /// runtime and belt carried on into it. Where they failed, everything made goes, the windows' swapchains before the
  /// device.
  ///
  /// # Errors
  ///
  /// Returns the build's error, or one saying it panicked.
  pub(crate) fn finish(self) -> XrfResult<RunningGpu> {
    let Self {
      surfaces,
      runtime,
      belt,
      build,
      ..
    } = self;
    let gpu: GpuState = build.join().unwrap_or_else(|_| {
      Err(XrfError::new_unexpected_error(
        "The renderer's passes panicked while built",
      ))
    })?;

    Ok(RunningGpu {
      frame_span: FrameSpanTimer::new(&gpu.context.device, &gpu.context.queue),
      runtime,
      belt,
      gpu,
      surfaces,
      is_level_cache_saved: false,
    })
  }

  /// Lets the GPU go: the windows' swapchains first, then the device once the passes' thread ended, so a backend
  /// started next finds every window free of them and no second device beside its own.
  pub(crate) fn release(mut self) {
    self.surfaces.swapchains.clear();

    if self.build.join().is_err() {
      log::error!("The renderer's passes panicked while built");
    }
  }
}
