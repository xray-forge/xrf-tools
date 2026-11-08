use std::error::Error;

use tauri::utils::config::WindowConfig;
use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::window::Color;
use tauri::{App, AppHandle, Manager, Runtime};
use xrf_app_core::preferences::Preferences;
use xrf_app_core::window::{WindowHandles, WindowReveals};
use xrf_build_info::BuildInfo;

use crate::window::geometry::{WindowGeometry, WindowGeometryTracker};
use crate::window::webview_extensions::WebviewExtensions;
use crate::window::window_build_kind::WindowBuildKind;

/// The window `tauri.conf.json` describes, which every page of the application runs in.
pub(crate) struct MainWindow;

impl MainWindow {
  /// Build the window and bring it up.
  ///
  /// # Errors
  ///
  /// Returns an error when `tauri.conf.json` declares no window, or the window cannot be built.
  pub(crate) fn build<R: Runtime>(application: &App<R>) -> Result<(), Box<dyn Error>> {
    let config: &WindowConfig = application
      .config()
      .app
      .windows
      .first()
      .ok_or("The main window has to be declared in tauri.conf.json")?;

    // A window nobody can open is worse than one that forgot where it was, or what it was set to run with.
    let preferences: Option<Preferences> = match Preferences::open(application.handle()) {
      Ok(preferences) => Some(preferences),
      Err(error) => {
        log::error!("Opening the main window without its saved preferences: {error}");

        None
      }
    };

    let opened: Option<WindowGeometry> = preferences
      .as_ref()
      .and_then(|preferences| WindowGeometryTracker::restore(application.handle(), preferences, config));

    let builder: WebviewWindowBuilder<'_, R, AppHandle<R>> =
      WebviewWindowBuilder::from_config(application.handle(), config)?
        .background_color(Self::to_see_through(config.background_color));
    let builder: WebviewWindowBuilder<'_, R, AppHandle<R>> = match &opened {
      Some(geometry) => geometry.open(builder),
      None => builder,
    };
    let window: WebviewWindow<R> =
      WindowBuildKind::apply(WebviewExtensions::apply(builder), application.state::<BuildInfo>().kind)?.build()?;

    log::info!("Built main window");

    if let Some(geometry) = &opened {
      geometry.confirm(&window);
    }

    // Recorded for the native viewports the window's pages attach, which name it by label.
    application.state::<WindowHandles>().register(&window);

    if let Some(preferences) = preferences {
      WindowGeometryTracker::track(&window, preferences, opened);
    }

    // The configuration declares it hidden: its page reveals it once it has something themed to show.
    application.state::<WindowReveals>().hold(window);

    Ok(())
  }

  /// The configured background with its alpha taken away, which only the webview honours: its pixels the page leaves
  /// transparent show the native viewport drawn on the window under it, and the window keeps the colour wherever no
  /// viewport draws.
  fn to_see_through(color: Option<Color>) -> Color {
    let Color(red, green, blue, _) = color.unwrap_or(Color(0, 0, 0, 255));

    Color(red, green, blue, 0)
  }
}
