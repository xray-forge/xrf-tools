use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::frame::frame_capture::CaptureReply;
use crate::host::render_event_sink::RenderEventSink;
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
  /// Names what a viewport draws under one of its points.
  Pick {
    id: RenderViewportId,
    pick: PendingPick,
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
  /// Finds a spawned object's bounding sphere in a viewport's level, none until its model is in the scene.
  LocateSpawnObject {
    id: RenderViewportId,
    object: u32,
    reply: Sender<Option<[f32; 4]>>,
  },
  /// Reads a viewport's next presented frame back.
  Capture {
    id: RenderViewportId,
    reply: CaptureReply,
  },
}
