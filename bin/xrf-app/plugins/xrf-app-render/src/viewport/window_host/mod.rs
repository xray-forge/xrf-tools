//! The native windows viewports are drawn into: one host for each platform the renderer presents on.

#[cfg(target_os = "macos")]
mod macos_window_host;
#[cfg(windows)]
mod win32_window_host;
mod window_host_builder;

pub use window_host_builder::WindowHostBuilder;
