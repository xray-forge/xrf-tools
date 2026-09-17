//! The main window: how it comes up, where it opens, and what puts it on screen.

mod main_window;
mod monitor_work_area;
mod window_geometry;
mod window_geometry_fit;
mod window_geometry_restore;
mod window_geometry_state;
mod window_geometry_tracker;
mod window_reveal;

#[cfg(test)]
mod tests;

pub use main_window::build_main_window;
