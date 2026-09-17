use crate::core::window::monitor_work_area::MonitorWorkArea;
use crate::core::window::window_geometry::WindowGeometry;

/// Read a saved rectangle against the monitors attached now.
pub fn fit_window_geometry(
  saved: WindowGeometry,
  monitors: &[MonitorWorkArea],
  min_width: f64,
  min_height: f64,
) -> Option<WindowGeometry> {
  if !saved.is_measured() {
    return None;
  }

  let target: MonitorWorkArea = monitors
    .iter()
    .copied()
    .map(|area| (measure_overlap(&saved, &area), area))
    .filter(|(overlap, _)| *overlap > 0)
    .max_by_key(|(overlap, _)| *overlap)
    .map(|(_, area)| area)?;

  // A monitor scaled differently from the one the rectangle was measured on shows the same window at another size.
  let ratio: f64 = target.scale / saved.scale;

  let width: u32 = fit_length(
    scale_length(saved.width, ratio),
    to_physical(min_width, target.scale),
    target.width,
  );
  let height: u32 = fit_length(
    scale_length(saved.height, ratio),
    to_physical(min_height, target.scale),
    target.height,
  );

  Some(WindowGeometry {
    x: fit_position(saved.x, width, target.x, target.width),
    y: fit_position(saved.y, height, target.y, target.height),
    width,
    height,
    scale: target.scale,
    maximized: saved.maximized,
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

fn measure_overlap(saved: &WindowGeometry, area: &MonitorWorkArea) -> i64 {
  let width: i64 = measure_overlap_length(saved.x, saved.width, area.x, area.width);
  let height: i64 = measure_overlap_length(saved.y, saved.height, area.y, area.height);

  width * height
}

fn measure_overlap_length(position: i32, length: u32, origin: i32, available: u32) -> i64 {
  let start: i64 = i64::from(position).max(i64::from(origin));
  let end: i64 = (i64::from(position) + i64::from(length)).min(i64::from(origin) + i64::from(available));

  (end - start).max(0)
}
