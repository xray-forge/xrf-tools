use std::path::PathBuf;
use std::sync::PoisonError;
use std::time::Instant;

use xrf_error::XrfResult;

use crate::context::gpu_context::GpuContext;
use crate::context::library_versions::locate_dlss_runtime;
use crate::contract::render_backend::RenderBackend;
use crate::thread::gpu_state::GpuState;
use crate::thread::retry_backoff::RetryBackoff;
use crate::thread::running_gpu::RunningGpu;

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
    let used: Vec<u64> = self.viewports.values().map(|viewport| viewport.window).collect();

    self.hosts.retain(|key, _| used.contains(key));
    self.last_present.retain(|key, _| used.contains(key));
    self.window_failures.retain(|key, _| used.contains(key));

    if let Some(RunningGpu { gpu, .. }) = &mut self.running {
      gpu.windows.retain(|key, _| used.contains(key));
    }

    if self.viewports.is_empty() && self.idle_since.is_none() {
      self.idle_since = Some(Instant::now());
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
    self.reopening.hold(
      self
        .viewports
        .values()
        .filter_map(|viewport| viewport.shown.as_ref().map(|(id, _)| *id)),
    );
    self
      .world
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .restart_scenes();

    for viewport in self.viewports.values_mut() {
      viewport.binding = None;
      viewport.show(None);
      viewport.release_captures();
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
      (dxc.as_deref(), dlss.as_deref()),
    )
    .and_then(|context| {
      let building: Instant = Instant::now();
      let built: XrfResult<GpuState> = GpuState::new(
        context,
        &self.shaders,
        &self.workers,
        (self.bundle.as_ref(), self.store.as_deref()),
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
