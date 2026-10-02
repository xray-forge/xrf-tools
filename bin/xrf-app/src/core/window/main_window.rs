use std::error::Error;

use tauri::utils::config::WindowConfig;
use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::{App, Manager, Runtime};
use xrf_build_info::build_info;

use crate::core::preferences::{PreferenceKey, Preferences};
use crate::core::webview_extensions::DevExtensions;
use crate::core::window::webview_options::WebviewOptions;
use crate::core::window::webview_options_state::{TWebviewOptionsStore, WebviewOptionsState};
use crate::core::window::window_build_kind::WindowBuildKind;
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

  // A window nobody can open is worse than one that forgot where it was, or what it was set to run with.
  let preferences: Option<Preferences<R>> = match Preferences::open(application.handle()) {
    Ok(preferences) => Some(preferences),
    Err(error) => {
      log::error!("Opening the main window without its saved preferences: {error}");

      None
    }
  };

  // Read before the webview exists: its browser is started once, with these, for the whole run.
  let webview_options: WebviewOptions = preferences
    .as_ref()
    .and_then(|preferences| preferences.read(PreferenceKey::WebviewOptions))
    .unwrap_or_default();

  application.manage(WebviewOptionsState::new(
    webview_options,
    to_webview_options_store(preferences.clone()),
  ));

  let window: WebviewWindow<R> = WebviewWindowBuilder::from_config(application.handle(), config)?
    .additional_browser_args(&webview_options.to_browser_args())
    .with_dev_extensions()
    .with_build_kind(build_info!().kind)?
    .build()?;

  log::info!("Built main window with webview options {webview_options:?}");

  if let Some(preferences) = preferences {
    let opened: Option<WindowGeometry> = restore_window_geometry(&window, &preferences, config);

    track_window_geometry(&window, preferences, opened);
  }

  // The configuration declares it hidden: the document reveals itself once it has resolved its colour scheme.
  reveal_window_on_timeout(window);

  Ok(())
}

/// Where a choice of webview options is kept for the next start: the preferences, written out at once.
fn to_webview_options_store<R: Runtime>(preferences: Option<Preferences<R>>) -> TWebviewOptionsStore {
  match preferences {
    Some(preferences) => Box::new(move |options: &WebviewOptions| {
      preferences.write(PreferenceKey::WebviewOptions, options)?;
      preferences.flush()
    }),
    None => Box::new(|_: &WebviewOptions| {
      Err("The preferences could not be opened this run, so the choice cannot be kept.".to_string())
    }),
  }
}
