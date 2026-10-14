use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use xrf_renderer::{
  RenderBundle, RenderEventSink, RenderLevelSource, RenderQueue, RenderSceneId, RenderViewportId, RenderWindowHost,
  RenderWorkers, RenderWorld,
};
use xrf_vfs::XrayRoots;
use xrf_world::{World, WorldEventSink};

use crate::plugins::render::disk_cache_store::DiskCacheStore;
use crate::plugins::render::renderer::Renderer;
use crate::plugins::render::texture_watcher::{TextureWatcher, to_texture_name};
use crate::plugins::render::viewport_windows::ViewportWindows;

/// How long a closing window waits for the render thread to let go of its swapchain: a frame as a rule.
const CLOSE_WAIT: Duration = Duration::from_secs(1);

/// How long the exit waits for the render thread to let its GPU go.
const STOP_WAIT: Duration = Duration::from_secs(5);

/// The application's one renderer, which starts a GPU only once a viewport is attached.
pub struct RenderState {
  pub renderer: Renderer,
  /// The world the renderer draws, which the camera and weather commands steer; its thread runs it each frame.
  world: Arc<Mutex<World>>,
  /// The latest show asked of each viewport, so one that took longer to prepare never replaces one asked after it.
  shows: Mutex<HashMap<RenderViewportId, u64>>,
  /// The window each viewport draws into, so a page that goes takes its viewports with it.
  windows: Mutex<ViewportWindows>,
  /// The shown sources' roots, watched for texture files written there.
  textures: TextureWatcher,
}

impl RenderState {
  /// A renderer whose loaders read and decode on `workers`, reading the files it ships with from `bundle` and keeping
  /// its compiled pipelines in `store`.
  pub fn new(workers: RenderWorkers, bundle: Arc<dyn RenderBundle>, store: Option<DiskCacheStore>) -> Self {
    let world: Arc<Mutex<World>> = Arc::new(Mutex::new(World::new(workers.clone())));
    let queue: RenderQueue = RenderQueue::new(workers, bundle, Arc::clone(&world) as Arc<Mutex<dyn RenderWorld>>);
    let renderer: Renderer = Renderer::new(match store {
      Some(store) => queue.with_cache_store(Arc::new(store)),
      None => queue,
    });

    Self {
      textures: TextureWatcher::new(renderer.get_queue()),
      renderer,
      world,
      shows: Mutex::default(),
      windows: Mutex::default(),
    }
  }

  /// Starts drawing a viewport into a window, named by its label, telling its page through `sink`; none once every
  /// viewport number was handed.
  pub fn attach_viewport(
    &self,
    window: &str,
    host: Arc<dyn RenderWindowHost>,
    (sink, world_sink): (Box<dyn RenderEventSink>, Box<dyn WorldEventSink>),
  ) -> Option<RenderViewportId> {
    let Some(viewport) = self.renderer.attach_viewport(host, sink) else {
      log::error!("No native viewport number is left to attach one to window '{window}'");

      return None;
    };

    self.lock_world().attach(viewport, world_sink);

    self.lock_windows().insert(viewport, window);

    log::info!("Attached native viewport {} to window '{window}'", viewport.0);

    Some(viewport)
  }

  /// Stops drawing a viewport its page let go.
  pub fn detach_viewport(&self, viewport: RenderViewportId) {
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

  /// Stops drawing every viewport in a window about to be destroyed, waiting until the render thread let go of its
  /// swapchain, so nothing is presented into a window that went.
  pub fn close_window(&self, window: &str) {
    if self.detach_window(window) && !self.renderer.wait_handled(CLOSE_WAIT) {
      log::warn!(
        "The renderer did not let go of window '{window}' within {} ms of its close",
        CLOSE_WAIT.as_millis()
      );
    }
  }

  /// Detaches every viewport and stops the renderer as the application exits, so its GPU is let go in order rather
  /// than with the process.
  pub fn shut_down(&self) {
    let viewports: Vec<RenderViewportId> = self.lock_windows().take_all();

    for viewport in viewports {
      self.release(viewport);
    }

    if !self.renderer.stop(STOP_WAIT) {
      log::warn!(
        "The renderer did not stop within {} ms of the exit",
        STOP_WAIT.as_millis()
      );
    }
  }

  /// A ticket for what a viewport is asked to show next, which every earlier ticket of it gives way to.
  pub fn ask_show(&self, viewport: RenderViewportId) -> u64 {
    let mut shows: MutexGuard<'_, HashMap<RenderViewportId, u64>> = self.lock_shows();
    let ticket: &mut u64 = shows.entry(viewport).or_default();

    *ticket += 1;
    *ticket
  }

  /// Shows a source in a viewport, none for `None`, unless something was asked of it after `ticket`; answers the
  /// scene shown.
  pub fn show(
    &self,
    viewport: RenderViewportId,
    ticket: u64,
    source: Option<Arc<dyn RenderLevelSource>>,
  ) -> Option<RenderSceneId> {
    let shows: MutexGuard<'_, HashMap<RenderViewportId, u64>> = self.lock_shows();

    if shows.get(&viewport) == Some(&ticket) {
      self.lock_world().show_level(viewport, source)
    } else {
      log::info!("Native viewport {} was asked to show something newer", viewport.0);

      None
    }
  }

  /// Watches the roots a shown source reads for texture files written there.
  pub fn watch_textures(&self, roots: &XrayRoots) {
    self.textures.watch(roots);
  }

  /// Loads again wherever they are drawn the textures among files this application wrote, by their paths.
  pub fn reload_written<T: AsRef<std::path::Path>>(&self, paths: &[T]) {
    let names: Vec<String> = paths.iter().filter_map(|path| to_texture_name(path.as_ref())).collect();

    self.renderer.reload_textures(names);
  }

  /// The world, locked for a command; the render thread holds it only while it runs a viewport's frame.
  pub fn lock_world(&self) -> MutexGuard<'_, World> {
    self.world.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn release(&self, viewport: RenderViewportId) {
    self.renderer.detach_viewport(viewport);
    self.lock_world().detach(viewport);
    self.lock_shows().remove(&viewport);
  }

  fn lock_shows(&self) -> MutexGuard<'_, HashMap<RenderViewportId, u64>> {
    self.shows.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn lock_windows(&self) -> MutexGuard<'_, ViewportWindows> {
    self.windows.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
