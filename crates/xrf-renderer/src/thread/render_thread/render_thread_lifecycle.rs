use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::PoisonError;
use std::time::Instant;

use xrf_error::XrfResult;

use crate::context::gpu_context::GpuContext;
use crate::context::gpu_libraries::GpuLibraries;
use crate::context::library_versions::locate_dlss_runtime;
use crate::contract::settings::gpu::render_backend::RenderBackend;
use crate::thread::gpu_state::GpuState;
use crate::thread::retry_backoff::RetryBackoff;
use crate::thread::running_gpu::RunningGpu;
use crate::viewport::render_viewport::RenderViewport;

use super::RenderThread;

impl RenderThread {
  /// Detaches viewports whose page stopped listening.
  pub(super) fn forget_gone(&mut self) {
    let before: usize = self.viewports.len();

    self.viewports.retain(|_, viewport| !viewport.is_gone());

    if self.viewports.len() != before {
      self.forget_unused_windows();
    }
  }

  /// Lets go of windows no viewport is drawn into, and starts the idle clock when none is left.
  pub(super) fn forget_unused_windows(&mut self) {
    let used: Vec<u64> = self.viewports.values().map(RenderViewport::get_window).collect();

    self.hosts.retain(|key, _| used.contains(key));
    self.last_present.retain(|key, _| used.contains(key));
    self.window_failures.retain(|key, _| used.contains(key));

    if let Some(RunningGpu { gpu, .. }) = &mut self.running {
      gpu.windows.retain(|key, _| used.contains(key));
    }

    self.start_idle_clock();
  }

  /// Starts the idle clock once no viewport draws and no hold keeps the GPU, unless it runs.
  pub(super) fn start_idle_clock(&mut self) {
    if self.viewports.is_empty() && self.holds.is_empty() && self.idle_since.is_none() {
      self.idle_since = Some(Instant::now());
    }
  }

  /// What a held thread with no viewport keeps: the GPU and its passes, ready for the next viewport, but none of what
  /// the last ones drew. The world keeps no scene no viewport shows, so each goes as the next frame would let it go, and
  /// the textures only they sampled with it; the frames' caches go too, which would otherwise hold the last view's
  /// targets alive; and the device is polled as a frame's report polls it, since wgpu frees what was dropped only then.
  pub(super) fn keep_held(&mut self, now: Instant) {
    self.retain_scenes(&HashSet::new());
    self.sweep_textures(now);

    if let Some(RunningGpu { gpu, runtime, .. }) = &mut self.running {
      runtime.forget_frames();
      let _ = gpu.context.device.poll(wgpu::PollType::Poll);
    }
  }

  /// Stops the thread unless a command arrived meanwhile: checked and decided under the link's lock, which every
  /// command is sent under. The GPU is let go after the lock, so no command waits on its teardown; its windows went
  /// with the last viewport.
  pub(super) fn try_stop(&mut self) -> bool {
    // No queue left to send anything: nothing can arrive, so the thread stops.
    let Some(link) = self.link.upgrade() else {
      if let Some(running) = self.running.take() {
        running.release(self.store.as_deref());
      }

      return true;
    };
    let mut link = link.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Ok(command) = self.receiver.try_recv() {
      drop(link);
      self.handle(command);

      return false;
    }

    link.sender = None;
    drop(link);

    if let Some(running) = self.running.take() {
      running.release(self.store.as_deref());
    }

    true
  }

  /// Lets the GPU go, with everything made on its device, for `ensure_gpu` to start it again.
  pub(super) fn release_gpu(&mut self) {
    if let Some(running) = self.running.take() {
      running.release(self.store.as_deref());
    }

    // What its allocator held went with it, so a GPU started again reports its own, its peak from nothing.
    self.allocator_report = None;

    // Its scenes went with it; the world streams each in again, from the start, into scenes made again, which the
    // viewports showing them wait for whole.
    self.scenes.clear();
    self
      .reopening
      .hold(self.viewports.values().filter_map(RenderViewport::get_shown_scene));
    self
      .world
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .restart_scenes();

    // The captures are told the GPU went before the viewport leaves its view, which would tell them it shows another
    // scene.
    for viewport in self.viewports.values_mut() {
      viewport.forget_binding();
      viewport.release_captures();
      viewport.hide();
    }
  }

  /// Starts the GPU when there is none, or again after it was lost, on the backend asked for, else the first that
  /// starts.
  pub(super) fn ensure_gpu(&mut self) {
    if self
      .running
      .as_ref()
      .is_some_and(|running| running.gpu.context.is_lost())
    {
      log::warn!("Restarting the renderer after its GPU was lost");
      self.release_gpu();
    }

    if self.running.is_some() || self.failure.as_ref().is_some_and(|failure| !failure.is_due()) {
      return;
    }

    let dxc: Option<PathBuf> = self.bundle.locate_bundled(GpuContext::DXC_LIBRARY);
    let dlss: Option<PathBuf> =
      locate_dlss_runtime(self.bundle.as_ref()).and_then(|library| library.parent().map(PathBuf::from));

    match GpuContext::create_preferred(
      RenderBackend::get_asked(self.settings.backend),
      self.settings.gpu,
      GpuLibraries {
        dxc: dxc.as_deref(),
        dlss: dlss.as_deref(),
      },
    )
    .and_then(|context| {
      let building: Instant = Instant::now();
      let built: XrfResult<GpuState> = GpuState::new(
        context,
        &self.shaders,
        &self.workers,
        self.bundle.as_ref(),
        self.store.as_deref(),
      );

      log::info!("Renderer's passes built in {} ms", building.elapsed().as_millis());

      built
    }) {
      Ok(gpu) => {
        self.running = Some(RunningGpu::new(gpu));
        self.failure = None;

        for viewport in self.viewports.values_mut() {
          viewport.recover();
        }
      }
      Err(error) => {
        let message: String = error.to_string();
        let is_new: bool = match &mut self.failure {
          Some(failure) => failure.fail(message.clone()),
          None => {
            self.failure = Some(RetryBackoff::new(message.clone()));

            true
          }
        };

        if is_new {
          log::error!("Renderer cannot start: {message}");
        }

        for viewport in self.viewports.values_mut() {
          viewport.fail(&message);
        }
      }
    }
  }
}
