use tauri::{Monitor, PhysicalRect};

/// The part of one monitor a window may occupy, with the taskbar and its kind already excluded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorWorkArea {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  pub scale: f64,
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
