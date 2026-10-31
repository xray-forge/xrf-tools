use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use xrf_error::XrfResult;

use crate::capture::burst::burst_event::BurstEvent;
use crate::capture::capture_event::CaptureEvent;
use crate::contract::render_burst::RenderBurst;
use crate::contract::render_capture::RenderCapture;
use crate::contract::render_cube_capture::RenderCubeCapture;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_map_capture::RenderMapCapture;
use crate::contract::render_memory_report::RenderMemoryReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_pick::RenderPick;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_window_host::RenderWindowHost;

/// What the render thread is told, through a [`crate::RenderQueue`]; it is the only one touching the GPU.
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
  Settings {
    settings: RenderSettings,
  },
  Options {
    id: RenderViewportId,
    options: Box<RenderViewOptions>,
  },
  /// Replaces what a viewport draws over its frame.
  Overlays {
    id: RenderViewportId,
    overlays: Vec<RenderOverlay>,
  },
  /// Marks what of a viewport's level is selected, or nothing for `None`.
  Selection {
    id: RenderViewportId,
    selection: Option<RenderSelection>,
  },
  /// Names what a viewport draws under one of its points, CSS pixels from its corner, answered after its next frame;
  /// the answer never comes for a viewport the renderer does not draw. `is_through` passes glass and every other
  /// composited surface by, naming what stands behind it.
  Pick {
    id: RenderViewportId,
    x: f32,
    y: f32,
    is_through: bool,
    reply: Sender<XrfResult<RenderPick>>,
  },
  /// Says what became of every texture a viewport's level samples; empty where it draws no level.
  DescribeTextures {
    id: RenderViewportId,
    reply: Sender<Vec<RenderTextureReport>>,
  },
  /// Says how far the level a viewport was last asked to show has loaded; none before its view is made.
  DescribeLoad {
    id: RenderViewportId,
    reply: Sender<Option<RenderLoadReport>>,
  },
  /// Says what a viewport's frames cost when it last reported them; none before its first report.
  DescribeFrame {
    id: RenderViewportId,
    reply: Sender<Option<RenderFrameReport>>,
  },
  /// Says what the renderer holds on the GPU and what the GPU's own memory holds of the process; none without a GPU.
  DescribeMemory {
    reply: Sender<Option<RenderMemoryReport>>,
  },
  /// Finds a spawned object's bounding sphere in a viewport's level, none until its model is in the scene.
  LocateSpawnObject {
    id: RenderViewportId,
    object: u32,
    reply: Sender<Option<[f32; 4]>>,
  },
  /// Loads again every texture of these file names (lowercase, without folder or extension), in every scene and
  /// weather: their files changed.
  ReloadTextures {
    names: Vec<String>,
  },
  /// Reads a viewport's next presented frame back; the answer never comes for a viewport the renderer does not draw.
  Capture {
    id: RenderViewportId,
    reply: Sender<XrfResult<RenderCapture>>,
  },
  /// Reads a viewport's next finished frame back whole, as presented but without its overlays or its selection's
  /// outline, whatever of it the window shows, its level objects' helpers drawn over it where `is_helped`; the answer
  /// never comes for a viewport the renderer does not draw.
  Screenshot {
    id: RenderViewportId,
    is_helped: bool,
    reply: Sender<XrfResult<RenderCapture>>,
  },
  /// Captures a level map in a viewport, in place of its own frames until it ends, telling `events` of each tile as it
  /// comes back; one capture at a time a viewport. Dropping the receiving end stops it, at its next word.
  CaptureMap {
    id: RenderViewportId,
    capture: RenderMapCapture,
    events: Sender<CaptureEvent>,
  },
  /// Captures a cubemap around a viewport's camera as `CaptureMap` captures a map, telling `events` of each face.
  CaptureCube {
    id: RenderViewportId,
    capture: RenderCubeCapture,
    events: Sender<CaptureEvent>,
  },
  /// Draws a burst of frames in a viewport through its own view, telling `events` of each frame as it comes back; one
  /// burst at a time over every viewport, as it paces the clock every scene moves by. Dropping the receiving end stops
  /// it, at its next word.
  CaptureBurst {
    id: RenderViewportId,
    burst: RenderBurst,
    events: Sender<BurstEvent>,
  },
  /// Draws every scene's frames `step` apart on its clock from the next on, however long each takes; none at the time
  /// each is drawn.
  StepClock {
    step: Option<Duration>,
  },
  /// Answers once every command sent before it is handled.
  Handled {
    reply: Sender<()>,
  },
  /// Detaches every viewport and stops the thread at once, answering once the GPU is let go; an attach sent after it
  /// keeps the thread running and lets the answer go unsent.
  Stop {
    reply: Sender<()>,
  },
}
