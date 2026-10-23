use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use xrf_renderer::{
  RenderBundle, RenderEventSink, RenderLevelSource, RenderQueue, RenderSceneId, RenderViewportId, RenderWait,
  RenderWindowHost, RenderWorkers, RenderWorld,
};
use xrf_vfs::XrayRoots;
use xrf_world::{World, WorldEventSink};

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{Session, SessionSnapshot};
use crate::core::types::TauriResult;

use crate::plugins::levels::render_source::LevelRenderSource;
use crate::plugins::levels::state::SelectedLevel;
use crate::plugins::render::disk_cache_store::DiskCacheStore;
use crate::plugins::render::level_scene_follower::LevelSceneFollower;
use crate::plugins::render::renderer::Renderer;
use crate::plugins::render::shown_levels::ShownLevels;
use crate::plugins::render::texture_watcher::{TextureWatcher, to_texture_name};
use crate::plugins::render::viewport_windows::ViewportWindows;

/// How long a closing window is held for the render thread to let go of its swapchain before it says so: a frame as a
/// rule.
const CLOSE_WARNING: Duration = Duration::from_secs(1);

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
  /// The windows held open until the render thread lets go of the viewports they drew, by label.
  closing: Mutex<HashSet<String>>,
  /// The shown sources' roots, watched for texture files written there.
  textures: TextureWatcher,
  /// The source each level session is drawn from, which a commit moves along.
  levels: Arc<ShownLevels>,
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
      closing: Mutex::default(),
      levels: Arc::default(),
    }
  }

  /// The source drawing a level's snapshot, shared by every viewport showing it.
  pub fn get_level_source(
    &self,
    level: Arc<SessionSnapshot<SelectedLevel>>,
    assets: &AssetMountState,
  ) -> Arc<LevelRenderSource> {
    self.levels.get_source(level, assets)
  }

  /// What moves the scene of each level shown along with `levels` through every commit.
  pub(crate) fn follow_levels(&self, levels: Session<SelectedLevel>) -> LevelSceneFollower {
    LevelSceneFollower {
      levels,
      shown: Arc::clone(&self.levels),
      world: Arc::clone(&self.world),
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

  /// Stops drawing every viewport in a window asked to close. One that drew any is held open until the render thread
  /// let go of its swapchain, then `destroy` is called on a thread of its own, so nothing is presented into a window
  /// that went and the event loop never waits; answers whether the close must be held. A window already held stays
  /// held.
  pub fn close_window(&self, window: &str, destroy: impl FnOnce() + Send + 'static) -> bool {
    let mut closing: MutexGuard<'_, HashSet<String>> = self.lock_closing();

    if closing.contains(window) {
      return true;
    }

    if !self.detach_window(window) {
      return false;
    }

    let (queue, label): (RenderQueue, String) = (self.renderer.get_queue(), window.to_owned());
    let held: std::io::Result<JoinHandle<()>> =
      std::thread::Builder::new()
        .name("xrf-window-close".into())
        .spawn(move || {
          Self::await_release(&queue, &label);
          destroy();
        });

    match held {
      Ok(_) => {
        closing.insert(window.to_owned());

        true
      }
      Err(error) => {
        log::error!("Window '{window}' cannot wait for the renderer off the event loop, so waits on it: {error}");
        Self::await_release(&self.renderer.get_queue(), window);

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

  /// Waits until the render thread handled every detach sent before, however long it takes, saying so once it takes
  /// longer than a frame should.
  fn await_release(queue: &RenderQueue, window: &str) {
    let mut wait: RenderWait = queue.wait_handled(CLOSE_WARNING);

    if wait == RenderWait::TimedOut {
      log::warn!(
        "Window '{window}' is held open until the renderer lets go of it, which takes over {} ms",
        CLOSE_WARNING.as_millis()
      );
      wait = queue.wait_handled(Duration::MAX);
    }

    if wait == RenderWait::Unanswered {
      log::error!("The renderer ended before it let go of window '{window}'");
    }
  }

  /// Detaches every viewport and stops the renderer as the application exits, so its GPU is let go in order rather
  /// than with the process.
  pub fn shut_down(&self) {
    let viewports: Vec<RenderViewportId> = self.lock_windows().take_all();

    for viewport in viewports {
      self.release(viewport);
    }

    match self.renderer.stop(STOP_WAIT) {
      RenderWait::Answered | RenderWait::NotRunning => {}
      RenderWait::Unanswered => log::warn!("The renderer did not answer its stop at the exit, attached again or ended"),
      RenderWait::TimedOut => log::warn!(
        "The renderer did not stop within {} ms of the exit",
        STOP_WAIT.as_millis()
      ),
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

  /// Loads again wherever they are drawn the textures among files this application wrote, by their paths: those
  /// outside the roots watched, as the watcher reloads the rest itself.
  pub fn reload_written<T: AsRef<std::path::Path>>(&self, paths: &[T]) {
    let names: Vec<String> = paths
      .iter()
      .map(AsRef::as_ref)
      .filter(|path| !self.textures.is_watched(path))
      .filter_map(to_texture_name)
      .collect();

    self.renderer.reload_textures(names);
  }

  /// The world, locked for a command; the render thread holds it while it advances every viewport and scene before a
  /// frame.
  pub fn lock_world(&self) -> MutexGuard<'_, World> {
    profiling::scope!("lock world");

    self.world.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// Runs `read` over the world on the execution pool, so a command waiting for the render thread's frame to let go of
  /// it holds no async worker meanwhile.
  pub async fn read_world<T, F>(&self, execution: &ExecutionState, what: &str, read: F) -> TauriResult<T>
  where
    F: FnOnce(&World) -> T + Send + 'static,
    T: Send + 'static,
  {
    let world: Arc<Mutex<World>> = Arc::clone(&self.world);

    execution
      .run_blocking(what, move || {
        read(&world.lock().unwrap_or_else(PoisonError::into_inner))
      })
      .await
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

  fn lock_closing(&self) -> MutexGuard<'_, HashSet<String>> {
    self.closing.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
