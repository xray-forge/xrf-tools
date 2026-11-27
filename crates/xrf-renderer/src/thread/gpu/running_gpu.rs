use wgpu::util::StagingBelt;
use xrf_renderer_core::GraphRuntime;

use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::timing::frame_span_timer::FrameSpanTimer;
use crate::host::render_cache_store::RenderCacheStore;
use crate::thread::gpu::gpu_state::GpuState;
use crate::thread::gpu::window_surfaces::WindowSurfaces;

/// The GPU while it runs, once its passes are built (`StartingGpu::finish`): everything made on it, with what the
/// frames keep between them on its device, all let go together.
pub(crate) struct RunningGpu {
  /// What the frames' graphs keep between frames, their passes' GPU timer among them.
  pub runtime: GraphRuntime,
  /// How long the GPU spends on each frame, which the periodic work's budget is taken from.
  pub frame_span: FrameSpanTimer,
  /// The staging belt the frames' own writes go through, kept between frames; out while a frame's encoder holds it.
  pub belt: Option<StagingBelt>,
  pub gpu: GpuState,
  /// The windows' swapchains and the layer pass, kept from the start.
  pub surfaces: WindowSurfaces,
  /// Whether the driver's pipelines were kept once a level first stood ready, by then holding most it will make.
  pub is_level_cache_saved: bool,
}

impl RunningGpu {
  /// Opens a frame's encoder over the belt kept from the last frame, or a new one.
  pub(crate) fn begin_encoder(&mut self) -> FrameEncoder {
    let device: &wgpu::Device = &self.gpu.context.device;
    let belt: StagingBelt = self.belt.take().unwrap_or_else(|| FrameEncoder::create_belt(device));

    FrameEncoder::new(device, belt)
  }

  /// Lets the GPU go: the device's work finished and the driver's pipelines kept, unless it was lost and has neither,
  /// and the windows' swapchains let go before the device, so a backend started next finds every window free of them.
  pub(crate) fn release(mut self, store: Option<&dyn RenderCacheStore>) {
    if !self.gpu.context.is_lost() {
      let _ = self.gpu.context.device.poll(wgpu::PollType::wait_indefinitely());
      self.gpu.save_driver_cache(store);
    }

    self.surfaces.swapchains.clear();
  }
}
