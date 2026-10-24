use tauri::webview::WebviewWindow;
use tauri::{Monitor, PhysicalRect, Runtime};

/// The part of one monitor a window may occupy, with the taskbar and its kind already excluded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorWorkArea {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  pub scale: f64,
}

impl MonitorWorkArea {
  /// The work areas of every monitor attached to the desktop `window` is on, none where they cannot be read.
  pub fn list<R: Runtime>(window: &WebviewWindow<R>) -> Vec<Self> {
    match window.available_monitors() {
      Ok(monitors) => monitors.iter().map(Self::from).collect(),
      Err(error) => {
        log::error!("Failed to read the attached monitors: {error}");

        Vec::new()
      }
    }
  }
}

impl From<&Monitor> for MonitorWorkArea {
  fn from(monitor: &Monitor) -> Self {
    let area: &PhysicalRect<i32, u32> = monitor.work_area();

    Self {
      x: area.position.x,
      y: area.position.y,
      width: area.size.width,
      height: area.size.height,
      scale: monitor.scale_factor(),
    }
  }
}
