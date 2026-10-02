use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_input_event::RenderInputEvent;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::contract::render_weather_control::RenderWeatherControl;
use crate::contract::render_weather_play::RenderWeatherPlay;
use crate::contract::render_weather_transition::RenderWeatherTransition;
use crate::frame::frame_capture::CaptureReply;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_window_host::RenderWindowHost;
use crate::viewport::pending_pick::PendingPick;

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
  Options {
    id: RenderViewportId,
    options: RenderViewOptions,
  },
  /// Names what a viewport draws under one of its points.
  Pick {
    id: RenderViewportId,
    pick: PendingPick,
  },
  /// Counts what each shader table entry of a viewport's level draws; empty where it draws no level.
  MeasureSurfaces {
    id: RenderViewportId,
    reply: Sender<Vec<RenderSurfaceGeometry>>,
  },
  /// Says what became of every texture a viewport's level samples; empty where it draws no level.
  DescribeTextures {
    id: RenderViewportId,
    reply: Sender<Vec<RenderTextureReport>>,
  },
  /// Reads a viewport's next presented frame back.
  Capture {
    id: RenderViewportId,
    reply: CaptureReply,
  },
  /// Draws a level in a viewport, or nothing for `None`.
  Level {
    id: RenderViewportId,
    source: Option<Arc<dyn RenderLevelSource>>,
  },
  /// Plays a weather in a viewport's level from now on.
  Weather {
    id: RenderViewportId,
    play: RenderWeatherPlay,
    transition: RenderWeatherTransition,
  },
  WeatherControl {
    id: RenderViewportId,
    control: RenderWeatherControl,
  },
  /// Plays a viewport's weather on from a time of day, ending the effect playing.
  WeatherSeek {
    id: RenderViewportId,
    time: f32,
  },
  /// Plays a weather effect over a viewport's cycle, or ends the one playing for `None`.
  WeatherEffect {
    id: RenderViewportId,
    name: Option<String>,
  },
}
