use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::PoisonError;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};

use super::RenderThread;
use crate::context::gpu_context::GpuContext;
use crate::context::gpu_libraries::GpuLibraries;
use crate::context::library_versions::locate_dlss_runtime;
use crate::contract::settings::gpu::render_backend::RenderBackend;
use crate::thread::gpu::running_gpu::RunningGpu;
use crate::thread::gpu::starting_gpu::StartingGpu;
use crate::thread::retry_backoff::RetryBackoff;
use crate::viewport::render_viewport::RenderViewport;

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

    if let Some(RunningGpu { surfaces, .. }) = &mut self.running {
      surfaces.swapchains.retain(|key, _| used.contains(key));
    }

    if let Some(StartingGpu { surfaces, .. }) = &mut self.starting {
      surfaces.swapchains.retain(|key, _| used.contains(key));
    }

    self.start_idle_clock();
  }

  /// Starts the idle clock, if not running, once no viewport or hold is left.
  pub(super) fn start_idle_clock(&mut self) {
    if self.viewports.is_empty() && self.holds.is_empty() && self.idle_since.is_none() {
      self.idle_since = Some(Instant::now());
    }
  }

  /// Keeps a held thread with no viewport at just the GPU and its passes: frees the unshown scenes, their textures and
  /// the frame caches, then polls the device, since wgpu frees dropped resources only on a poll.
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
      self.let_gpu_go();

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
    self.let_gpu_go();

    true
  }

  /// Lets the GPU go, running or starting, with everything made on its device; a start waits for its passes' thread.
  fn let_gpu_go(&mut self) {
    if let Some(running) = self.running.take() {
      running.release(self.store.as_deref());
    }

    if let Some(starting) = self.starting.take() {
      starting.release();
    }
  }

  /// The device drawn on, running or starting; none before it starts.
  pub(super) fn get_context(&self) -> Option<&GpuContext> {
    match (&self.running, &self.starting) {
      (Some(running), _) => Some(&running.gpu.context),
      (None, Some(starting)) => Some(&starting.context),
      (None, None) => None,
    }
  }

  /// Lets the GPU go, with everything made on its device, for `ensure_gpu` to start it again.
  pub(super) fn release_gpu(&mut self) {
    self.let_gpu_go();

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

    // Captures are released before `hide`, which would report the viewport as showing another scene.
    for viewport in self.viewports.values_mut() {
      viewport.forget_binding();
      viewport.get_layer_mut().release_gpu();
      viewport.release_captures();
      viewport.hide();
    }
  }

  /// Starts the GPU when there is none, or again after it was lost, on the backend asked for, else the first that
  /// starts: its device at once, its passes on a thread of their own, taken once they are built.
  pub(super) fn ensure_gpu(&mut self) {
    if self.get_context().is_some_and(GpuContext::is_lost) {
      log::warn!("Restarting the renderer after its GPU was lost");
      self.release_gpu();
    }

    if self.starting.as_ref().is_some_and(StartingGpu::is_built) {
      self.finish_start();
    }

    if self.running.is_some()
      || self.starting.is_some()
      || self.failure.as_ref().is_some_and(|failure| !failure.is_due())
    {
      return;
    }

    let dxc: Option<PathBuf> = self.bundle.locate_bundled(GpuContext::DXC_LIBRARY);
    let dlss: Option<PathBuf> =
      locate_dlss_runtime(self.bundle.as_ref()).and_then(|library| library.parent().map(PathBuf::from));
    let started: XrfResult<StartingGpu> = GpuContext::create_preferred(
      RenderBackend::get_asked(self.settings.backend),
      self.settings.gpu,
      GpuLibraries {
        dxc: dxc.as_deref(),
        dlss: dlss.as_deref(),
      },
    )
    .and_then(|context| StartingGpu::begin(context, &self.shaders, &self.workers, &self.bundle, self.store.as_ref()));

    match started {
      Ok(starting) => self.starting = Some(starting),
      Err(error) => self.fail_start(&error),
    }
  }

  /// Takes the passes the start built and runs the GPU with them, or, where they failed, lets the start go with the
  /// layers' textures made on its device, to retry after a backoff.
  fn finish_start(&mut self) {
    let Some(starting) = self.starting.take() else {
      return;
    };

    match starting.finish() {
      Ok(running) => {
        self.running = Some(running);
        self.failure = None;

        for viewport in self.viewports.values_mut() {
          viewport.recover();
        }
      }
      Err(error) => {
        for viewport in self.viewports.values_mut() {
          viewport.get_layer_mut().release_gpu();
        }

        self.fail_start(&error);
      }
    }
  }

  /// Notes that the GPU failed to start, logged once until the reason changes and told to every viewport, and backs
  /// off before the next try.
  fn fail_start(&mut self, error: &XrfError) {
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
