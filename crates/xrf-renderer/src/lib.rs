//! The native renderer: one GPU and one render thread drawing viewports into the application's windows.
//!
//! It takes what it draws in memory and never opens a file, and knows nothing of the application hosting it beyond
//! [`RenderWindowHost`] and [`RenderEventSink`].

pub(crate) mod camera;
pub(crate) mod context;
pub(crate) mod contract;
pub(crate) mod frame;
pub(crate) mod host;
pub(crate) mod pass;
pub(crate) mod renderer;
pub(crate) mod shader;
pub(crate) mod thread;
pub(crate) mod viewport;
pub(crate) mod window;

#[cfg(test)]
mod tests;

pub use crate::context::render_backend::RenderBackend;
pub use crate::contract::render_camera::RenderCamera;
pub use crate::contract::render_camera_command::RenderCameraCommand;
pub use crate::contract::render_camera_pose::RenderCameraPose;
pub use crate::contract::render_color::RenderColor;
pub use crate::contract::render_frame_report::RenderFrameReport;
pub use crate::contract::render_input_event::RenderInputEvent;
pub use crate::contract::render_input_kind::RenderInputKind;
pub use crate::contract::render_presentation::RenderPresentation;
pub use crate::contract::render_rect::RenderRect;
pub use crate::contract::render_settings::RenderSettings;
pub use crate::contract::render_viewport_event::RenderViewportEvent;
pub use crate::contract::render_viewport_id::RenderViewportId;
pub use crate::contract::render_viewport_layout::RenderViewportLayout;
pub use crate::host::render_event_sink::RenderEventSink;
pub use crate::host::render_window_host::RenderWindowHost;
pub use crate::renderer::Renderer;
pub use raw_window_handle;
