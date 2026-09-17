use crate::core::window::monitor_work_area::MonitorWorkArea;
use crate::core::window::window_geometry::WindowGeometry;

/// The primary monitor of a plain desktop, with room taken by a taskbar along the bottom.
pub const PRIMARY: MonitorWorkArea = MonitorWorkArea {
  x: 0,
  y: 0,
  width: 1920,
  height: 1040,
  scale: 1.0,
};

/// A second monitor to the left, as Windows addresses one: negative coordinates in the same desktop space.
pub const LEFT: MonitorWorkArea = MonitorWorkArea {
  x: -1280,
  y: 0,
  width: 1280,
  height: 1024,
  scale: 1.0,
};

/// The minimum `tauri.conf.json` declares, in the logical pixels it declares it in.
pub const MIN_WIDTH: f64 = 900.0;
pub const MIN_HEIGHT: f64 = 600.0;

/// A rectangle measured on a monitor that does not scale.
pub fn geometry(x: i32, y: i32, width: u32, height: u32) -> WindowGeometry {
  WindowGeometry {
    x,
    y,
    width,
    height,
    scale: 1.0,
    maximized: false,
  }
}
