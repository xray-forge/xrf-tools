use serde::{Deserialize, Serialize};
use tauri::webview::WebviewWindow;
use tauri::{PhysicalPosition, PhysicalSize, Runtime};

use crate::window::geometry::monitor_work_area::MonitorWorkArea;

/// Where the main window was last left, in the physical pixels the desktop addresses its monitors with.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowGeometry {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  /// Scale factor of the monitor the rectangle was measured on, which decides how it is read on a different one.
  pub scale: f64,
  pub maximized: bool,
}

impl Default for WindowGeometry {
  /// The rectangle no window has: every field a partial file leaves out reads as unmeasured.
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
      scale: 0.0,
      maximized: false,
    }
  }
}

impl WindowGeometry {
  /// Whether the rectangle describes a window at all, which a hand-edited or truncated file need not.
  pub fn is_measured(&self) -> bool {
    self.width > 0 && self.height > 0 && self.scale.is_finite() && self.scale > 0.0
  }

  /// Read the rectangle against the monitors attached now, as a window no smaller than `min_width` by `min_height`
  /// logical pixels.
  pub fn fit(self, monitors: &[MonitorWorkArea], min_width: f64, min_height: f64) -> Option<Self> {
    if !self.is_measured() {
      return None;
    }

    let target: MonitorWorkArea = monitors
      .iter()
      .copied()
      .map(|area| (self.measure_overlap(&area), area))
      .filter(|(overlap, _)| *overlap > 0)
      .max_by_key(|(overlap, _)| *overlap)
      .map(|(_, area)| area)?;

    // A monitor scaled differently from the one the rectangle was measured on shows the same window at another size.
    let ratio: f64 = target.scale / self.scale;

    let width: u32 = Self::fit_length(
      Self::scale_length(self.width, ratio),
      Self::to_physical(min_width, target.scale),
      target.width,
    );
    let height: u32 = Self::fit_length(
      Self::scale_length(self.height, ratio),
      Self::to_physical(min_height, target.scale),
      target.height,
    );

    Some(Self {
      x: Self::fit_position(self.x, width, target.x, target.width),
      y: Self::fit_position(self.y, height, target.y, target.height),
      width,
      height,
      scale: target.scale,
      maximized: self.maximized,
    })
  }

  /// Read the window's own rectangle, keeping `previous` for what a window in that state cannot report.
  pub fn measure<R: Runtime>(window: &WebviewWindow<R>, previous: Self) -> Self {
    let is_maximized: bool = Self::read_window_state(window.is_maximized(), "maximized");

    // A maximized or minimized window occupies a rectangle nobody chose; the one it restores down to is already held.
    if is_maximized || Self::read_window_state(window.is_minimized(), "minimized") {
      return Self {
        maximized: is_maximized,
        ..previous
      };
    }

    let (position, size, scale) = match Self::measure_rectangle(window) {
      Ok(measured) => measured,
      Err(error) => {
        log::error!("Failed to measure the main window: {error}");

        return previous;
      }
    };

    Self {
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
      scale,
      maximized: false,
    }
  }

  /// The three measurements together, so one unreadable window is one failure rather than three.
  fn measure_rectangle<R: Runtime>(
    window: &WebviewWindow<R>,
  ) -> tauri::Result<(PhysicalPosition<i32>, PhysicalSize<u32>, f64)> {
    Ok((window.outer_position()?, window.inner_size()?, window.scale_factor()?))
  }

  fn read_window_state(state: tauri::Result<bool>, name: &str) -> bool {
    state.unwrap_or_else(|error| {
      log::error!("Failed to read whether the main window is {name}: {error}");

      false
    })
  }

  /// Physical length of a size the window configuration states in logical pixels.
  fn to_physical(length: f64, scale: f64) -> u32 {
    let physical: f64 = (length * scale).round();

    if physical > 0.0 {
      physical.min(f64::from(u32::MAX)) as u32
    } else {
      0
    }
  }

  fn scale_length(length: u32, ratio: f64) -> u32 {
    (f64::from(length) * ratio).round().clamp(1.0, f64::from(u32::MAX)) as u32
  }

  /// Clamps a side between the size the window refuses to go below and the work area that has to hold it.
  fn fit_length(length: u32, minimum: u32, available: u32) -> u32 {
    length.clamp(minimum.min(available), available)
  }

  /// Slides a side back inside the work area, keeping its origin when the window is as wide as the area itself.
  fn fit_position(position: i32, length: u32, origin: i32, available: u32) -> i32 {
    let remainder: i32 = i32::try_from(available.saturating_sub(length)).unwrap_or(i32::MAX);
    let last: i32 = origin.saturating_add(remainder);

    position.clamp(origin, origin.max(last))
  }

  fn measure_overlap(&self, area: &MonitorWorkArea) -> i64 {
    let width: i64 = Self::measure_overlap_length(self.x, self.width, area.x, area.width);
    let height: i64 = Self::measure_overlap_length(self.y, self.height, area.y, area.height);

    width * height
  }

  fn measure_overlap_length(position: i32, length: u32, origin: i32, available: u32) -> i64 {
    let start: i64 = i64::from(position).max(i64::from(origin));
    let end: i64 = (i64::from(position) + i64::from(length)).min(i64::from(origin) + i64::from(available));

    (end - start).max(0)
  }
}
