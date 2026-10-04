use std::error::Error;

use tauri::utils::config::WindowConfig;
use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::window::Color;
use tauri::{App, Manager, Runtime};
use xrf_build_info::build_info;

use crate::core::preferences::Preferences;
use crate::core::webview_extensions::DevExtensions;
use crate::core::window::window_build_kind::WindowBuildKind;
use crate::core::window::window_geometry::WindowGeometry;
use crate::core::window::window_geometry_restore::restore_window_geometry;
use crate::core::window::window_geometry_tracker::track_window_geometry;
use crate::core::window::window_handles::WindowHandles;
use crate::core::window::window_reveal::reveal_window_on_timeout;

/// Build the window `tauri.conf.json` describes and bring it up.
pub fn build_main_window<R: Runtime>(application: &App<R>) -> Result<(), Box<dyn Error>> {
  let config: &WindowConfig = application
    .config()
    .app
    .windows
    .first()
    .expect("Main window has to be declared in tauri.conf.json");

  // A window nobody can open is worse than one that forgot where it was, or what it was set to run with.
  let preferences: Option<Preferences<R>> = match Preferences::open(application.handle()) {
    Ok(preferences) => Some(preferences),
    Err(error) => {
      log::error!("Opening the main window without its saved preferences: {error}");

      None
    }
  };

  let window: WebviewWindow<R> = WebviewWindowBuilder::from_config(application.handle(), config)?
    .background_color(to_see_through(config.background_color))
    .with_dev_extensions()
    .with_build_kind(build_info!().kind)?
    .build()?;

  log::info!("Built main window");

  // Recorded for the native viewports the window's pages attach, which name it by label.
  #[cfg(windows)]
  if let Some(handle) = window
    .hwnd()
    .ok()
    .and_then(|hwnd| std::num::NonZeroIsize::new(hwnd.0 as isize))
  {
    application.state::<WindowHandles>().register(window.label(), handle);
  }

  if let Some(preferences) = preferences {
    let opened: Option<WindowGeometry> = restore_window_geometry(&window, &preferences, config);

    track_window_geometry(&window, preferences, opened);
  }

  // The configuration declares it hidden: the document reveals itself once it has resolved its colour scheme.
  reveal_window_on_timeout(window);

  Ok(())
}

/// The configured background with its alpha taken away, which only the webview honours: its pixels the page leaves
/// transparent show the native viewport drawn on the window under it, and the window keeps the colour wherever no
/// viewport draws.
fn to_see_through(color: Option<Color>) -> Color {
  let Color(red, green, blue, _) = color.unwrap_or(Color(0, 0, 0, 255));

  Color(red, green, blue, 0)
}
