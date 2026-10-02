use std::sync::Arc;

use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_input_event::RenderInputEvent;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_window_host::RenderWindowHost;

/// What the render thread is told; it is the only one touching the GPU.
pub enum RenderCommand {
  Attach {
    id: RenderViewportId,
    host: Arc<dyn RenderWindowHost>,
    sink: Box<dyn RenderEventSink>,
  },
  Detach {
    id: RenderViewportId,
  },
  Layout {
    id: RenderViewportId,
    layout: RenderViewportLayout,
  },
  Input {
    id: RenderViewportId,
    event: RenderInputEvent,
  },
  Camera {
    id: RenderViewportId,
    camera: RenderCamera,
  },
  CameraCommand {
    id: RenderViewportId,
    command: RenderCameraCommand,
  },
  Settings {
    settings: RenderSettings,
  },
}
