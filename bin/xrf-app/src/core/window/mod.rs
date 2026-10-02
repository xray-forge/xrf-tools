//! The main window: how it comes up, where it opens, what its webview runs with, and what puts it on screen.

mod main_window;
mod monitor_work_area;
mod webview_collection_pace;
mod webview_options;
mod webview_options_state;
mod window_build_kind;
mod window_geometry;
mod window_geometry_fit;
mod window_geometry_restore;
mod window_geometry_state;
mod window_geometry_tracker;
mod window_handles;
mod window_reveal;

#[cfg(test)]
mod tests;

pub use main_window::build_main_window;
pub use webview_collection_pace::WebviewCollectionPace;
pub use webview_options::WebviewOptions;
pub use webview_options_state::WebviewOptionsState;
pub use window_handles::WindowHandles;
