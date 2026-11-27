use std::io::ErrorKind;
use std::sync::Arc;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_platform::{
  Platform, PlatformSize, PlatformWaker, PlatformWindow, PlatformWindowHandle, PlatformWindowMode,
  PlatformWindowRequest,
};
use xrf_game_settings::{GameSettings, GameWindowMode};
use xrf_renderer::RenderWorkers;
use xrf_renderer_host::HostedWindow;

use crate::directories::game_directories::GameDirectories;
use crate::render::game_view::GameView;

/// What the game is played in, started on the main thread as SDL requires and needing no level: the platform, the
/// window (hidden until its first frame) and the renderer drawing into it. It starts while the session opens, so the
/// renderer's own start (its device, DLSS, its passes) runs beside reading the level rather than after it.
pub(crate) struct GameShell {
  pub(crate) platform: Platform,
  pub(crate) window: PlatformWindow,
  hosted: HostedWindow<PlatformWindowHandle>,
  pub(crate) view: GameView,
}

impl GameShell {
  /// Opens the window titled for `level` as `settings` set it, and starts the renderer in it on `pool`'s workers. The
  /// platform takes `waker`, so the threads holding it end the main thread's wait for events from here on, the
  /// renderer's among them.
  ///
  /// # Errors
  ///
  /// Returns why SDL, the window or its host cannot start.
  pub(crate) fn start(
    level: &str,
    settings: &GameSettings,
    directories: &GameDirectories,
    pool: &Arc<ThreadPool>,
    waker: &PlatformWaker,
  ) -> XrfResult<Self> {
    let mut platform: Platform = Platform::start().map_err(Self::create_platform_error)?;

    platform.attach(waker.clone());

    let window: PlatformWindow = platform
      .open_window(&PlatformWindowRequest {
        title: format!("XRF: {level}"),
        size: PlatformSize {
          width: settings.window.width,
          height: settings.window.height,
        },
        mode: match settings.window.mode {
          GameWindowMode::Windowed => PlatformWindowMode::Windowed,
          GameWindowMode::Fullscreen => PlatformWindowMode::Fullscreen,
        },
      })
      .map_err(Self::create_platform_error)?;
    let hosted: HostedWindow<PlatformWindowHandle> = window.create_hosted().map_err(Self::create_platform_error)?;
    let view: GameView = GameView::start(
      RenderWorkers::new(Arc::clone(pool)),
      settings,
      directories.get_cache().as_ref(),
    );

    view.attach(hosted.get_host(), &window, settings, waker.to_waker());

    Ok(Self {
      platform,
      window,
      hosted,
      view,
    })
  }

  /// Stops the renderer and closes the window, in the order the window's drop needs: the view, its host, the window.
  pub(crate) fn stop(self) {
    let Self {
      platform,
      window,
      hosted,
      view,
    } = self;

    view.stop();
    drop(hosted);
    drop(window);
    drop(platform);
  }

  fn create_platform_error(message: String) -> XrfError {
    XrfError::new_io_error(message, ErrorKind::Other)
  }
}
