//! Where the main window opens: the rectangle the last run left, fitted to the monitors attached now, and followed so
//! the next run can do the same.

mod monitor_work_area;
mod window_geometry;
mod window_geometry_state;
mod window_geometry_tracker;

#[cfg(test)]
mod tests;

pub use window_geometry::WindowGeometry;
pub use window_geometry_tracker::WindowGeometryTracker;
