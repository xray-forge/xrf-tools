use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, channel};

use xrf_error::XrfResult;
use xrf_renderer::{
  RenderCapture, RenderCommand, RenderEventSink, RenderFrameReport, RenderLoadReport, RenderOverlay, RenderPick,
  RenderQueue, RenderSelection, RenderSettings, RenderTextureReport, RenderViewOptions, RenderViewportId,
  RenderViewportLayout, RenderWindowHost,
};

/// The renderer as the application calls it: each call a command on the render thread's queue, numbering the
/// viewports it attaches and answering a question through a channel of its own.
pub struct Renderer {
  queue: RenderQueue,
  next_id: AtomicU32,
}

impl Renderer {
  pub fn new(queue: RenderQueue) -> Self {
    Self {
      queue,
      next_id: AtomicU32::new(0),
    }
  }

  /// Starts drawing a viewport into a window, answering the number every later call names it by.
  pub fn attach_viewport(&self, host: Arc<dyn RenderWindowHost>, sink: Box<dyn RenderEventSink>) -> RenderViewportId {
    let id: RenderViewportId = RenderViewportId(self.next_id.fetch_add(1, Ordering::Relaxed) + 1);

    self.queue.send(RenderCommand::Attach { id, host, sink });

    id
  }

  pub fn detach_viewport(&self, id: RenderViewportId) {
    self.queue.send(RenderCommand::Detach { id });
  }

  pub fn set_layout(&self, id: RenderViewportId, layout: RenderViewportLayout) {
    self.queue.send(RenderCommand::Layout { id, layout });
  }

  /// What a viewport's level draws under a point, css pixels from its corner, answered after its next frame; the
  /// answer never comes for a viewport the renderer does not draw.
  pub fn pick(&self, id: RenderViewportId, x: f32, y: f32) -> Receiver<XrfResult<RenderPick>> {
    let (reply, answer) = channel();

    self.queue.send(RenderCommand::Pick { id, x, y, reply });

    answer
  }

  /// What became of every texture a viewport's level samples, answered at once.
  pub fn describe_textures(&self, id: RenderViewportId) -> Receiver<Vec<RenderTextureReport>> {
    let (reply, answer) = channel();

    self.queue.send(RenderCommand::DescribeTextures { id, reply });

    answer
  }

  /// How far the level a viewport was last asked to show has loaded, answered at once, for a caller polling rather than
  /// listening: its `Load` events report the level drawn, which is still the one before while another loads. None
  /// before that level's view is made, or where it shows no level.
  pub fn describe_load(&self, id: RenderViewportId) -> Receiver<Option<RenderLoadReport>> {
    let (reply, answer) = channel();

    self.queue.send(RenderCommand::DescribeLoad { id, reply });

    answer
  }

  /// What a viewport's frames cost when it last reported them, answered at once: what its `Frame` events report, for a
  /// caller polling rather than listening. None before its first report.
  pub fn describe_frame(&self, id: RenderViewportId) -> Receiver<Option<RenderFrameReport>> {
    let (reply, answer) = channel();

    self.queue.send(RenderCommand::DescribeFrame { id, reply });

    answer
  }

  /// A spawned object's bounding sphere in renderer space, centre then radius, answered at once; none until its model
  /// is in the scene.
  pub fn locate_spawn_object(&self, id: RenderViewportId, object: u32) -> Receiver<Option<[f32; 4]>> {
    let (reply, answer) = channel();

    self.queue.send(RenderCommand::LocateSpawnObject { id, object, reply });

    answer
  }

  /// A viewport's next presented frame, read back; the answer never comes for a viewport the renderer does not draw.
  pub fn capture_viewport(&self, id: RenderViewportId) -> Receiver<XrfResult<RenderCapture>> {
    let (reply, answer) = channel();

    self.queue.send(RenderCommand::Capture { id, reply });

    answer
  }

  pub fn set_view_options(&self, id: RenderViewportId, options: RenderViewOptions) {
    self.queue.send(RenderCommand::Options {
      id,
      options: Box::new(options),
    });
  }

  /// Replaces what a viewport draws over its frame.
  pub fn set_overlays(&self, id: RenderViewportId, overlays: Vec<RenderOverlay>) {
    self.queue.send(RenderCommand::Overlays { id, overlays });
  }

  /// Marks what of a viewport's level is selected, or nothing for `None`.
  pub fn set_selection(&self, id: RenderViewportId, selection: Option<RenderSelection>) {
    self.queue.send(RenderCommand::Selection { id, selection });
  }

  /// Applies settings to every viewport, now and in every thread started later.
  pub fn configure(&self, settings: RenderSettings) {
    self.queue.send(RenderCommand::Settings { settings });
  }
}
