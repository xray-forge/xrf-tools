use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use xrf_cache::AppCache;
use xrf_error::XrfResult;
use xrf_game_developer::DeveloperFrame;
use xrf_game_platform::{PlatformSize, PlatformWindow};
use xrf_game_settings::GameSettings;
use xrf_renderer::{
  RenderCapture, RenderCommand, RenderFrameReport, RenderLoadReport, RenderQueue, RenderRect, RenderViewportId,
  RenderViewportLayout, RenderWait, RenderWindowHost, RenderWorkers, RenderWorld,
};
use xrf_renderer_host::DiskCacheStore;
use xrf_world::{World, WorldLevelSource, WorldSoundCue, WorldViewFeed};

use crate::render::game_bundle::GameBundle;
use crate::render::game_render_sink::GameRenderSink;
use crate::render::game_world_sink::GameWorldSink;

/// The game's one view of the world: the render queue and the world it draws, and the viewport filling the window.
pub struct GameView {
  queue: RenderQueue,
  world: Arc<Mutex<World>>,
  /// Whether the level shown stood ready, as the renderer reported it.
  ready: Arc<AtomicBool>,
  /// How far the level shown loaded, as the renderer last reported it.
  load: Arc<Mutex<Option<RenderLoadReport>>>,
  /// How many frames of the view the renderer drew.
  drawn: Arc<AtomicU64>,
  /// The latest frame report not yet taken.
  frame: Arc<Mutex<Option<Box<RenderFrameReport>>>>,
}

impl GameView {
  /// The viewport that fills the game's window.
  pub const VIEWPORT: RenderViewportId = RenderViewportId(1);

  /// How long stopping waits on the render thread at each turn.
  const STOP_WAIT: Duration = Duration::from_secs(5);

  /// The renderer and its world over `workers`, drawn as `settings` set it; the pipeline cache kept in `cache`.
  pub fn start(workers: RenderWorkers, settings: &GameSettings, cache: Option<&AppCache>) -> Self {
    let world: Arc<Mutex<World>> = Arc::new(Mutex::new(World::new(workers.clone())));
    let mut queue: RenderQueue = RenderQueue::new(
      workers,
      Arc::new(GameBundle::locate()),
      Arc::clone(&world) as Arc<Mutex<dyn RenderWorld>>,
    );

    if let Some(cache) = cache {
      queue = queue.with_cache_store(Arc::new(DiskCacheStore::new(cache)));
    }

    queue.send(RenderCommand::Settings {
      settings: settings.render,
    });

    Self {
      queue,
      world,
      ready: Arc::new(AtomicBool::new(false)),
      load: Arc::new(Mutex::new(None)),
      drawn: Arc::new(AtomicU64::new(0)),
      frame: Arc::new(Mutex::new(None)),
    }
  }

  /// How many frames of the view the renderer drew so far.
  pub fn count_drawn(&self) -> u64 {
    self.drawn.load(Ordering::Acquire)
  }

  /// How far the level shown loaded, none before the renderer first reports it.
  pub fn get_load(&self) -> Option<RenderLoadReport> {
    *self.load.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// The frame report the renderer sent since the last taken, none where it sent none.
  pub fn take_frame_report(&self) -> Option<Box<RenderFrameReport>> {
    self.frame.lock().unwrap_or_else(PoisonError::into_inner).take()
  }

  /// Whether the level shown stood ready: every sector, particle and texture settled.
  pub fn is_ready(&self) -> bool {
    self.ready.load(Ordering::Acquire)
  }

  /// Draws the view into `window` through the `host` it presents to, with the view's features and upscaling from
  /// `settings`.
  pub fn attach(&self, host: Arc<dyn RenderWindowHost>, window: &PlatformWindow, settings: &GameSettings) {
    self.queue.send(RenderCommand::Attach {
      id: Self::VIEWPORT,
      host,
      sink: Box::new(GameRenderSink {
        ready: Arc::clone(&self.ready),
        load: Arc::clone(&self.load),
        drawn: Arc::clone(&self.drawn),
        frame: Arc::clone(&self.frame),
      }),
    });
    self.lock_world().attach(Self::VIEWPORT, Box::new(GameWorldSink));
    self.send_options(settings);
    self.fit(window);
  }

  /// Draws with `settings` from the next frame: what every frame draws with, and the view's features and upscaling. The
  /// window's own settings apply only as it opens.
  pub fn apply_settings(&self, settings: &GameSettings) {
    self.queue.send(RenderCommand::Settings {
      settings: settings.render,
    });
    self.send_options(settings);
  }

  /// The view's features and upscaling from `settings`.
  fn send_options(&self, settings: &GameSettings) {
    self.queue.send(RenderCommand::Options {
      id: Self::VIEWPORT,
      options: Box::new(settings.get_view_options()),
    });
  }

  /// Fits the view to `window` as it stands: its drawable size in device pixels, and how many of them a logical pixel
  /// spans.
  pub fn fit(&self, window: &PlatformWindow) {
    let PlatformSize { width, height } = window.get_size();

    self.queue.send(RenderCommand::Layout {
      id: Self::VIEWPORT,
      layout: RenderViewportLayout {
        rect: RenderRect {
          x: 0,
          y: 0,
          width,
          height,
        },
        scale: window.get_scale(),
        ..RenderViewportLayout::default()
      },
    });
  }

  /// Asks for the window's next finished frame read back, answered on the receiver; none comes while the window draws
  /// nothing.
  pub fn capture(&self) -> Receiver<XrfResult<RenderCapture>> {
    let (reply, answer) = mpsc::channel();

    self.queue.send(RenderCommand::Capture {
      id: Self::VIEWPORT,
      reply,
    });

    answer
  }

  /// Draws the developer layer's `frame` over the view from the next frame on, its texture changes applied first.
  pub fn draw_layer(&self, frame: DeveloperFrame) {
    let DeveloperFrame { textures, layer } = frame;

    if !textures.set.is_empty() || !textures.free.is_empty() {
      self.queue.send(RenderCommand::LayerTextures {
        id: Self::VIEWPORT,
        textures,
      });
    }

    self.queue.send(RenderCommand::Layer {
      id: Self::VIEWPORT,
      layer,
    });
  }

  /// Shows `level`, its camera and the game's day given by `feed`.
  pub fn show(&self, level: Arc<dyn WorldLevelSource>, feed: Arc<dyn WorldViewFeed>) {
    let mut world: MutexGuard<'_, World> = self.lock_world();

    world.show_level(Self::VIEWPORT, Some(level));
    world.set_view_feed(Self::VIEWPORT, feed);
  }

  /// Plays `cue` in the shown level, at its next sound frame.
  pub fn play_sound(&self, cue: WorldSoundCue) {
    self.lock_world().play_sound(Self::VIEWPORT, cue);
  }

  /// Takes the view out of the window and stops the renderer, in the order the window's drop needs: detached, the
  /// detach handled, the render thread stopped. The window may be dropped after.
  pub fn stop(self) {
    self.queue.send(RenderCommand::Detach { id: Self::VIEWPORT });
    self.lock_world().detach(Self::VIEWPORT);

    let handled: RenderWait = self.queue.wait_handled(Self::STOP_WAIT);
    let stopped: RenderWait = self.queue.stop(Self::STOP_WAIT);

    log::info!("Renderer stopped: detach {handled:?}, thread {stopped:?}");
  }

  fn lock_world(&self) -> MutexGuard<'_, World> {
    self.world.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
