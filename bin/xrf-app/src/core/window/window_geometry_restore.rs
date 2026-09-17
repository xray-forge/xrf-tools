use tauri::utils::config::WindowConfig;
use tauri::webview::WebviewWindow;
use tauri::{PhysicalPosition, PhysicalSize, Runtime};

use crate::core::preferences::{PreferenceKey, Preferences};
use crate::core::window::monitor_work_area::MonitorWorkArea;
use crate::core::window::window_geometry::WindowGeometry;
use crate::core::window::window_geometry_fit::fit_window_geometry;

/// Open the window where the last run left it, when the monitors attached now still have room for it.
pub fn restore_window_geometry<R: Runtime>(
  window: &WebviewWindow<R>,
  preferences: &Preferences<R>,
  config: &WindowConfig,
) -> Option<WindowGeometry> {
  let saved: WindowGeometry = preferences.read(PreferenceKey::WindowGeometry)?;
  let fitted: WindowGeometry = fit_window_geometry(
    saved,
    &resolve_work_areas(window),
    config.min_width.unwrap_or_default(),
    config.min_height.unwrap_or_default(),
  )?;

  if let Err(error) = window.set_size(PhysicalSize::new(fitted.width, fitted.height)) {
    log::error!("Failed to open the main window at its saved size: {error}");

    return None;
  }

  if let Err(error) = window.set_position(PhysicalPosition::new(fitted.x, fitted.y)) {
    log::error!("Failed to open the main window at its saved position: {error}");

    return None;
  }

  // After the rectangle rather than before it: this is what the window restores down to.
  if fitted.maximized
    && let Err(error) = window.maximize()
  {
    log::error!("Failed to open the main window maximized: {error}");
  }

  log::info!(
    "Restored main window geometry: {}x{} at {},{}{}",
    fitted.width,
    fitted.height,
    fitted.x,
    fitted.y,
    if fitted.maximized { ", maximized" } else { "" }
  );

  Some(fitted)
}

fn resolve_work_areas<R: Runtime>(window: &WebviewWindow<R>) -> Vec<MonitorWorkArea> {
  match window.available_monitors() {
    Ok(monitors) => monitors.iter().map(MonitorWorkArea::from).collect(),
    Err(error) => {
      log::error!("Failed to read the attached monitors: {error}");

      Vec::new()
    }
  }
}
