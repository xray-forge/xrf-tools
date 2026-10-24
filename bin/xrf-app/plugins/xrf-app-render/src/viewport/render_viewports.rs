use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tauri::ipc::Channel;
use xrf_renderer::{RenderQueue, RenderViewportId, RenderWait, RenderWindowHost};

use crate::viewport::channel_event_sink::ChannelEventSink;
use crate::viewport::viewport_windows::ViewportWindows;
use crate::{Renderer, ViewportEvent, ViewportShows, WorldState};

/// The native viewports drawn into the application's windows, attached to the renderer and the world together and let
/// go of together, as their page asks, as it goes or its window closes, or as the application exits.
pub struct RenderViewports {
  renderer: Renderer,
  world: WorldState,
  shows: ViewportShows,
  /// The window each viewport draws into, so a page that goes takes its viewports with it.
  windows: Mutex<ViewportWindows>,
  /// The windows held open until the render thread lets go of the viewports they drew, by label; shared with the thread
  /// waiting for each, which lets its window go once the renderer did.
  closing: Arc<Mutex<HashSet<String>>>,
}

impl RenderViewports {
  /// How long a closing window is held for the render thread to let go of its swapchain before it says so: a frame as a
  /// rule.
  const CLOSE_WARNING: Duration = Duration::from_secs(1);

  /// How long the exit waits for the render thread to let its GPU go.
  const STOP_WAIT: Duration = Duration::from_secs(5);

  /// The viewports `renderer` draws, showing what `shows` hand `world`.
  pub fn new(renderer: Renderer, world: WorldState, shows: ViewportShows) -> Self {
    Self {
      renderer,
      world,
      shows,
      windows: Mutex::default(),
      closing: Arc::default(),
    }
  }

  /// Starts drawing a viewport into a window, named by its label, telling its page through `events` what the renderer
  /// and the world say of it; none once every viewport number was handed.
  pub fn attach(
    &self,
    window: &str,
    host: Arc<dyn RenderWindowHost>,
    events: Channel<ViewportEvent>,
  ) -> Option<RenderViewportId> {
    let Some(viewport) = self
      .renderer
      .attach_viewport(host, Box::new(ChannelEventSink::new(events.clone())))
    else {
      log::error!("No native viewport number is left to attach one to window '{window}'");

      return None;
    };

    self
      .world
      .lock()
      .attach(viewport, Box::new(ChannelEventSink::new(events)));

    self.lock_windows().insert(viewport, window);

    log::info!("Attached native viewport {} to window '{window}'", viewport.0);

    Some(viewport)
  }

  /// Stops drawing a viewport its page let go.
  pub fn detach(&self, viewport: RenderViewportId) {
    let window: Option<String> = self.lock_windows().remove(viewport);

    match window {
      Some(window) => log::info!("Detached native viewport {} from window '{window}'", viewport.0),
      None => log::info!("Detached native viewport {}, which no window held", viewport.0),
    }

    self.release(viewport);
  }

  /// Stops drawing every viewport in a window whose page is loading again or which is going, neither of which can let
  /// them go itself; answers whether it held any.
  pub fn detach_window(&self, window: &str) -> bool {
    let viewports: Vec<RenderViewportId> = self.lock_windows().take_window(window);

    for viewport in &viewports {
      log::info!(
        "Detached native viewport {} from window '{window}', whose page went",
        viewport.0
      );

      self.release(*viewport);
    }

    !viewports.is_empty()
  }

  /// Stops drawing every viewport in a window asked to close. One that drew any is held open until the render thread
  /// let go of its swapchain, then `destroy` is called on a thread of its own, so nothing is presented into a window
  /// that went and the event loop never waits; answers whether the close must be held. A window already held stays
  /// held until the renderer let go of it, and is held no longer then, whatever its destroy answers, so a window that
  /// failed to go closes when asked again.
  pub fn close_window(&self, window: &str, destroy: impl FnOnce() + Send + 'static) -> bool {
    {
      let mut closing: MutexGuard<'_, HashSet<String>> = self.lock_closing();

      if closing.contains(window) {
        return true;
      }

      if !self.detach_window(window) {
        return false;
      }

      closing.insert(window.to_owned());
    }

    let queue: RenderQueue = self.renderer.get_queue();
    let closing: Arc<Mutex<HashSet<String>>> = Arc::clone(&self.closing);
    let label: String = window.to_owned();
    let held = std::thread::Builder::new()
      .name("xrf-window-close".into())
      .spawn(move || {
        Self::await_release(&queue, &label, Duration::MAX);
        closing.lock().unwrap_or_else(PoisonError::into_inner).remove(&label);
        destroy();
      });

    match held {
      Ok(_) => true,
      Err(error) => {
        log::error!("Window '{window}' cannot wait for the renderer off the event loop, so waits on it: {error}");
        self.lock_closing().remove(window);
        // Bounded, since the event loop is what waits: past it the window goes whether the renderer let go or not.
        Self::await_release(&self.renderer.get_queue(), window, Self::CLOSE_WARNING);

        false
      }
    }
  }

  /// Whether a window is held open until the render thread lets go of it.
  pub fn is_closing(&self, window: &str) -> bool {
    self.lock_closing().contains(window)
  }

  /// Forgets a destroyed window. One still drawing viewports went without the close that lets the renderer go of it
  /// first, which the window host's contract forbids: they are detached, and the breach logged.
  pub fn forget_window(&self, window: &str) {
    self.lock_closing().remove(window);

    if self.detach_window(window) {
      log::error!("Window '{window}' was destroyed while native viewports drew into it, before the renderer let it go");
    }
  }

  /// Detaches every viewport and stops the renderer as the application exits, so its GPU is let go in order rather
  /// than with the process.
  pub fn shut_down(&self) {
    let viewports: Vec<RenderViewportId> = self.lock_windows().take_all();

    for viewport in viewports {
      self.release(viewport);
    }

    match self.renderer.stop(Self::STOP_WAIT) {
      RenderWait::Answered | RenderWait::NotRunning => {}
      RenderWait::Unanswered => log::warn!("The renderer did not answer its stop at the exit, attached again or ended"),
      RenderWait::TimedOut => log::warn!(
        "The renderer did not stop within {} ms of the exit",
        Self::STOP_WAIT.as_millis()
      ),
    }
  }

  /// Waits until the render thread handled every detach sent before, `CLOSE_WARNING` and then `limit` more, saying so
  /// once it takes longer than a frame should.
  fn await_release(queue: &RenderQueue, window: &str, limit: Duration) {
    let mut wait: RenderWait = queue.wait_handled(Self::CLOSE_WARNING);

    if wait == RenderWait::TimedOut {
      log::warn!(
        "Window '{window}' is held open until the renderer lets go of it, which takes over {} ms",
        Self::CLOSE_WARNING.as_millis()
      );
      wait = queue.wait_handled(limit);
    }

    match wait {
      RenderWait::Unanswered => log::error!("The renderer ended before it let go of window '{window}'"),
      RenderWait::TimedOut => log::error!("Window '{window}' goes before the renderer let go of it"),
      RenderWait::Answered | RenderWait::NotRunning => {}
    }
  }

  fn release(&self, viewport: RenderViewportId) {
    self.renderer.detach_viewport(viewport);
    self.world.lock().detach(viewport);
    self.shows.forget(viewport);
  }

  fn lock_windows(&self) -> MutexGuard<'_, ViewportWindows> {
    self.windows.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn lock_closing(&self) -> MutexGuard<'_, HashSet<String>> {
    self.closing.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
