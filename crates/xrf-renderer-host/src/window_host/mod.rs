//! The native windows viewports are drawn into: one host for each platform the renderer presents on.

#[cfg(target_os = "macos")]
pub(crate) mod macos_window_host;
#[cfg(windows)]
pub(crate) mod win32_window_host;
pub(crate) mod window_host_builder;
