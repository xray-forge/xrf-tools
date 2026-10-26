//! The main window: how it comes up, where it opens, and what puts it on screen.

mod geometry;
mod main_window;
mod webview_extensions;
mod window_build_kind;

#[cfg(test)]
mod tests;

pub use main_window::MainWindow;
