use std::error::Error;

use tauri::utils::config::WindowConfig;
use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::{App, Runtime};

use crate::core::preferences::Preferences;
use crate::core::webview_extensions::DevExtensions;
use crate::core::window::window_geometry::WindowGeometry;
use crate::core::window::window_geometry_restore::restore_window_geometry;
use crate::core::window::window_geometry_tracker::track_window_geometry;
use crate::core::window::window_reveal::reveal_window_on_timeout;

/// Build the window `tauri.conf.json` describes and bring it up.
pub fn build_main_window<R: Runtime>(application: &App<R>) -> Result<(), Box<dyn Error>> {
  let config: &WindowConfig = application
    .config()
    .app
    .windows
    .first()
    .expect("Main window has to be declared in tauri.conf.json");

  let window: WebviewWindow<R> = WebviewWindowBuilder::from_config(application.handle(), config)?
    .with_dev_extensions()
    .build()?;

  // A window nobody can open is worse than one that forgot where it was.
  match Preferences::open(application.handle()) {
    Ok(preferences) => {
      let opened: Option<WindowGeometry> = restore_window_geometry(&window, &preferences, config);

      track_window_geometry(&window, preferences, opened);
    }
    Err(error) => log::error!("Opening the main window without its saved geometry: {error}"),
  }

  // The configuration declares it hidden: the document reveals itself once it has resolved its colour scheme.
  reveal_window_on_timeout(window);

  log::info!("Built main window");

  Ok(())
}
