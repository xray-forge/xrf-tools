use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use xrf_app_core::TauriResult;
use xrf_error::XrfResult;
use xrf_renderer::{
  BurstEvent, CaptureEvent, RenderBurst, RenderCapture, RenderCommand, RenderCubeCapture, RenderEventSink,
  RenderFrameReport, RenderHoldId, RenderLevelHit, RenderLibraries, RenderLoadReport, RenderMapCapture,
  RenderMemoryReport, RenderOverlay, RenderPick, RenderQueue, RenderSceneId, RenderScreenshot, RenderSelection,
  RenderSettings, RenderTextureReport, RenderViewOptions, RenderViewportId, RenderViewportLayout, RenderWait,
  RenderWindowHost,
};

use crate::renderer::{RenderAnswer, RenderPickRequest};

/// The renderer as the application calls it: each call a command on the render thread's queue, numbering the
/// viewports it attaches and answering a question through a channel of its own, a viewport's waited on for
/// `RenderAnswer::TIMEOUT`. A clone calls the same renderer.
#[derive(Clone)]
pub struct Renderer {
  queue: RenderQueue,
  /// The number the last viewport was attached under, none before the first.
  last_id: Arc<AtomicU32>,
  /// The number the last hold was taken under, none before the first.
  last_hold: Arc<AtomicU32>,
}

impl Renderer {
  /// The renderer whose render thread `queue` feeds, no viewport attached yet.
  pub fn new(queue: RenderQueue) -> Self {
    Self {
      queue,
      last_id: Arc::new(AtomicU32::new(0)),
      last_hold: Arc::new(AtomicU32::new(0)),
    }
  }

  /// Keeps the GPU started, starting it where it is not, answering the number the hold is released by; none once every
  /// number was handed. `RenderViewports` holds it for a window.
  pub(crate) fn hold(&self) -> Option<RenderHoldId> {
    let last: u32 = self
      .last_hold
      .try_update(Ordering::Relaxed, Ordering::Relaxed, |last| last.checked_add(1))
      .ok()?;
    let id: RenderHoldId = RenderHoldId(last + 1);

    self.queue.send(RenderCommand::Hold { id });

    Some(id)
  }

  /// Lets a hold go; the GPU goes a few seconds after the last hold and viewport.
  pub(crate) fn release_hold(&self, id: RenderHoldId) {
    self.queue.send(RenderCommand::Release { id });
  }

  /// Starts drawing a viewport into a window, answering the number every later call names it by; none once every
  /// number was handed, since a number is never handed twice. `RenderViewports` attaches the world with it.
  pub(crate) fn attach_viewport(
    &self,
    host: Arc<dyn RenderWindowHost>,
    sink: Box<dyn RenderEventSink>,
  ) -> Option<RenderViewportId> {
    let last: u32 = self
      .last_id
      .try_update(Ordering::Relaxed, Ordering::Relaxed, |last| last.checked_add(1))
      .ok()?;
    let id: RenderViewportId = RenderViewportId(last + 1);

    self.queue.send(RenderCommand::Attach { id, host, sink });

    Some(id)
  }

  /// Stops drawing a viewport, letting go of its swapchain on the render thread. `RenderViewports` detaches the world
  /// and its shows with it.
  pub(crate) fn detach_viewport(&self, id: RenderViewportId) {
    self.queue.send(RenderCommand::Detach { id });
  }

  /// Waits up to `wait` until the render thread handled every command sent before.
  pub(crate) fn wait_handled(&self, wait: Duration) -> RenderWait {
    self.queue.wait_handled(wait)
  }

  /// Detaches every viewport and stops the render thread, waiting up to `wait` until it let the GPU go.
  pub fn stop(&self, wait: Duration) -> RenderWait {
    self.queue.stop(wait)
  }

  /// Places a viewport within its window and sizes it.
  pub fn set_viewport_layout(&self, id: RenderViewportId, layout: RenderViewportLayout) {
    self.queue.send(RenderCommand::Layout { id, layout });
  }

  /// What a viewport's level draws under the point `request` names, answered after its next frame; a viewport the
  /// renderer does not draw, or one showing nothing, answers no hit at once.
  ///
  /// # Errors
  ///
  /// Returns an error when the viewport answers no pick in time, or the pick failed.
  pub async fn pick(&self, id: RenderViewportId, request: RenderPickRequest) -> TauriResult<Option<RenderLevelHit>> {
    let RenderPickRequest { x, y, is_through } = request;
    let answer: Receiver<XrfResult<RenderPick>> = self.ask(|reply| RenderCommand::Pick {
      id,
      x,
      y,
      is_through,
      reply,
    });

    RenderAnswer::await_viewport(answer, id, "pick")
      .await?
      .map(|pick| pick.hit)
      .map_err(|error| error.to_string())
  }

  /// What became of every texture a viewport's level samples, answered at once.
  ///
  /// # Errors
  ///
  /// Returns an error when the viewport answers nothing in time.
  pub async fn describe_texture_loads(&self, id: RenderViewportId) -> TauriResult<Vec<RenderTextureReport>> {
    let answer: Receiver<Vec<RenderTextureReport>> = self.ask(|reply| RenderCommand::DescribeTextures { id, reply });

    RenderAnswer::await_viewport(answer, id, "texture loads").await
  }

  /// How far the level a viewport was last asked to show has loaded, answered at once, for a caller polling rather than
  /// listening: its `Load` events report the level drawn, which is still the one before while another loads. None
  /// before that level's view is made, or where it shows no level.
  ///
  /// # Errors
  ///
  /// Returns an error when the viewport answers nothing in time.
  pub async fn describe_load(&self, id: RenderViewportId) -> TauriResult<Option<RenderLoadReport>> {
    let answer: Receiver<Option<RenderLoadReport>> = self.ask(|reply| RenderCommand::DescribeLoad { id, reply });

    RenderAnswer::await_viewport(answer, id, "load").await
  }

  /// What a viewport's frames cost when it last reported them, answered at once: what its `Frame` events report, for a
  /// caller polling rather than listening. None before its first report.
  ///
  /// # Errors
  ///
  /// Returns an error when the viewport answers nothing in time.
  pub async fn describe_frame(&self, id: RenderViewportId) -> TauriResult<Option<RenderFrameReport>> {
    let answer: Receiver<Option<RenderFrameReport>> = self.ask(|reply| RenderCommand::DescribeFrame { id, reply });

    RenderAnswer::await_viewport(answer, id, "frame").await
  }

  /// What the renderer holds on the GPU and what the GPU's own memory holds of the process, answered at once; the reply
  /// is let go unanswered while no render thread runs, and none while it runs without a GPU.
  pub fn describe_memory(&self) -> Receiver<Option<RenderMemoryReport>> {
    self.ask(|reply| RenderCommand::DescribeMemory { reply })
  }

  /// The graphics libraries the renderer ships with.
  pub fn describe_libraries(&self) -> RenderLibraries {
    self.queue.describe_libraries()
  }

  /// A spawned object's bounding sphere in renderer space, centre then radius, answered at once; none until its model
  /// is in the scene.
  ///
  /// # Errors
  ///
  /// Returns an error when the viewport answers nothing in time.
  pub async fn locate_spawn_object(&self, id: RenderViewportId, object: u32) -> TauriResult<Option<[f32; 4]>> {
    let answer: Receiver<Option<[f32; 4]>> = self.ask(|reply| RenderCommand::LocateSpawnObject { id, object, reply });

    RenderAnswer::await_viewport(answer, id, "location").await
  }

  /// A viewport's next presented frame, read back; the reply is let go unanswered for a viewport the renderer does not
  /// draw.
  pub fn capture_viewport(&self, id: RenderViewportId) -> Receiver<XrfResult<RenderCapture>> {
    self.ask(|reply| RenderCommand::Capture { id, reply })
  }

  /// A viewport's next finished frame whole, drawn as `screenshot` asks, read back; the reply is let go unanswered for a
  /// viewport the renderer does not draw.
  pub fn screenshot_viewport(
    &self,
    id: RenderViewportId,
    screenshot: RenderScreenshot,
  ) -> Receiver<XrfResult<RenderCapture>> {
    self.ask(|reply| RenderCommand::Screenshot { id, screenshot, reply })
  }

  /// Captures a level map in a viewport, answering what it tells as it goes; dropping the answer stops it.
  pub fn capture_map(&self, id: RenderViewportId, capture: RenderMapCapture) -> Receiver<CaptureEvent> {
    self.ask(|events| RenderCommand::CaptureMap { id, capture, events })
  }

  /// Captures a cubemap around a viewport's camera, answering what it tells as it goes; dropping the answer stops it.
  pub fn capture_cube(&self, id: RenderViewportId, capture: RenderCubeCapture) -> Receiver<CaptureEvent> {
    self.ask(|events| RenderCommand::CaptureCube { id, capture, events })
  }

  /// Draws a burst of frames in a viewport through its own view, answering each frame as it comes back; dropping the
  /// answer stops it.
  pub fn capture_burst(&self, id: RenderViewportId, burst: RenderBurst) -> Receiver<BurstEvent> {
    self.ask(|events| RenderCommand::CaptureBurst { id, burst, events })
  }

  /// Draws every scene's frames `step` apart on its clock from the next on; none at the time each is drawn.
  pub fn step_clock(&self, step: Option<Duration>) {
    self.queue.send(RenderCommand::StepClock { step });
  }

  /// Sets how a viewport draws what it shows: its mode, its passes and what it overlays.
  pub fn set_view_options(&self, id: RenderViewportId, options: RenderViewOptions) {
    self.queue.send(RenderCommand::Options {
      id,
      options: Box::new(options),
    });
  }

  /// Replaces what a viewport draws over its frame: at once, or from the frame it shows the scene `scene` names.
  pub fn set_overlays(&self, id: RenderViewportId, overlays: Vec<RenderOverlay>, scene: Option<u32>) {
    self.queue.send(RenderCommand::Overlays {
      id,
      overlays,
      scene: scene.map(RenderSceneId),
    });
  }

  /// Marks what of a viewport's level is selected, or nothing for `None`.
  pub fn set_selection(&self, id: RenderViewportId, selection: Option<RenderSelection>) {
    self.queue.send(RenderCommand::Selection { id, selection });
  }

  /// Loads again every texture of these file names (lowercase, without folder or extension) wherever it is drawn: their
  /// files changed.
  pub fn reload_textures(&self, names: Vec<String>) {
    if !names.is_empty() {
      self.queue.send(RenderCommand::ReloadTextures { names });
    }
  }

  /// The way to the render thread, for what tells it things from a thread of its own.
  pub(crate) fn get_queue(&self) -> RenderQueue {
    self.queue.clone()
  }

  /// Applies settings to every viewport, now and in every thread started later.
  pub fn configure(&self, settings: RenderSettings) {
    self.queue.send(RenderCommand::Settings { settings });
  }

  /// Sends the command `ask` makes around a reply channel, answering the channel's other end.
  fn ask<T>(&self, ask: impl FnOnce(Sender<T>) -> RenderCommand) -> Receiver<T> {
    let (reply, answer) = channel();

    self.queue.send(ask(reply));

    answer
  }
}
