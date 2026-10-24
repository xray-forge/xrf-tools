use std::error::Error;

use tauri::utils::config::WindowConfig;
use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::window::Color;
use tauri::{App, Manager, Runtime};
use xrf_app_core::window::WindowHandles;
use xrf_build_info::BuildInfo;

use crate::preferences::Preferences;
use crate::window::geometry::{WindowGeometry, WindowGeometryTracker};
use crate::window::webview_extensions::DevExtensions;
use crate::window::window_build_kind::WindowBuildKind;
use crate::window::window_reveal::WindowReveal;

/// The window `tauri.conf.json` describes, which every page of the application runs in.
pub struct MainWindow;

impl MainWindow {
  /// Build the window and bring it up.
  ///
  /// # Errors
  ///
  /// Returns an error when `tauri.conf.json` declares no window, or the window cannot be built.
  pub fn build<R: Runtime>(application: &App<R>) -> Result<(), Box<dyn Error>> {
    let config: &WindowConfig = application
      .config()
      .app
      .windows
      .first()
      .ok_or("The main window has to be declared in tauri.conf.json")?;

    // A window nobody can open is worse than one that forgot where it was, or what it was set to run with.
    let preferences: Option<Preferences<R>> = match Preferences::open(application.handle()) {
      Ok(preferences) => Some(preferences),
      Err(error) => {
        log::error!("Opening the main window without its saved preferences: {error}");

        None
      }
    };

    let window: WebviewWindow<R> = WebviewWindowBuilder::from_config(application.handle(), config)?
      .background_color(Self::to_see_through(config.background_color))
      .with_dev_extensions()
      .with_build_kind(application.state::<BuildInfo>().kind)?
      .build()?;

    log::info!("Built main window");

    // Recorded for the native viewports the window's pages attach, which name it by label.
    application.state::<WindowHandles>().register(&window);

    if let Some(preferences) = preferences {
      let opened: Option<WindowGeometry> = WindowGeometryTracker::restore(&window, &preferences, config);

      WindowGeometryTracker::track(&window, preferences, opened);
    }

    // The configuration declares it hidden: the document reveals itself once it has resolved its colour scheme.
    WindowReveal::on_timeout(window);

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
