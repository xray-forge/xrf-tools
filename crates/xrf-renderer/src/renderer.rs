use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};

use xrf_error::XrfResult;

use crate::contract::render_capture::RenderCapture;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_pick::RenderPick;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_window_host::RenderWindowHost;
use crate::host::render_world::RenderWorld;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_thread::RenderThread;
use crate::thread::render_workers::RenderWorkers;
use crate::viewport::pending_pick::PendingPick;

/// The renderer as an application holds it: cheap to make, and touching no GPU until a viewport is attached.
///
/// Every call is a message to the render thread, started by the first attach and stopped a few seconds after the
/// last detach, so a tool that never draws never starts a GPU.
pub struct Renderer {
  link: Arc<Mutex<RenderLink>>,
  next_id: AtomicU32,
  /// The pool every thread it starts reads and decodes on.
  workers: RenderWorkers,
  /// The files it ships with, which the application reads for it.
  bundle: Arc<dyn RenderBundle>,
  /// The world it draws, run on its thread before each frame.
  world: Arc<Mutex<dyn RenderWorld>>,
}

impl Renderer {
  pub fn new(workers: RenderWorkers, bundle: Arc<dyn RenderBundle>, world: Arc<Mutex<dyn RenderWorld>>) -> Self {
    Self {
      link: Arc::default(),
      next_id: AtomicU32::new(0),
      workers,
      bundle,
      world,
    }
  }

  /// Starts drawing a viewport into a window, answering the number every later call names it by.
  pub fn attach_viewport(&self, host: Arc<dyn RenderWindowHost>, sink: Box<dyn RenderEventSink>) -> RenderViewportId {
    let id: RenderViewportId = RenderViewportId(self.next_id.fetch_add(1, Ordering::Relaxed) + 1);

    self.send(RenderCommand::Attach { id, host, sink });

    id
  }

  pub fn detach_viewport(&self, id: RenderViewportId) {
    self.send(RenderCommand::Detach { id });
  }

  pub fn set_layout(&self, id: RenderViewportId, layout: RenderViewportLayout) {
    self.send(RenderCommand::Layout { id, layout });
  }

  /// What a viewport's level draws under a point, css pixels from its corner, answered after its next frame; the
  /// answer never comes for a viewport the renderer does not draw.
  pub fn pick(&self, id: RenderViewportId, x: f32, y: f32) -> Receiver<XrfResult<RenderPick>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::Pick {
      id,
      pick: PendingPick { x, y, reply },
    });

    answer
  }

  /// What became of every texture a viewport's level samples, answered at once.
  pub fn describe_textures(&self, id: RenderViewportId) -> Receiver<Vec<RenderTextureReport>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::DescribeTextures { id, reply });

    answer
  }

  /// How far the level a viewport was last asked to show has loaded, answered at once, for a caller polling rather than
  /// listening: its `Load` events report the level drawn, which is still the one before while another loads. None
  /// before that level's view is made, or where it shows no level.
  pub fn describe_load(&self, id: RenderViewportId) -> Receiver<Option<RenderLoadReport>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::DescribeLoad { id, reply });

    answer
  }

  /// What a viewport's frames cost when it last reported them, answered at once: what its `Frame` events report, for a
  /// caller polling rather than listening. None before its first report.
  pub fn describe_frame(&self, id: RenderViewportId) -> Receiver<Option<RenderFrameReport>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::DescribeFrame { id, reply });

    answer
  }

  /// A spawned object's bounding sphere in renderer space, centre then radius, answered at once; none until its model
  /// is in the scene.
  pub fn locate_spawn_object(&self, id: RenderViewportId, object: u32) -> Receiver<Option<[f32; 4]>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::LocateSpawnObject { id, object, reply });

    answer
  }

  /// A viewport's next presented frame, read back; the answer never comes for a viewport the renderer does not draw.
  pub fn capture_viewport(&self, id: RenderViewportId) -> Receiver<XrfResult<RenderCapture>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::Capture { id, reply });

    answer
  }

  pub fn set_view_options(&self, id: RenderViewportId, options: RenderViewOptions) {
    self.send(RenderCommand::Options {
      id,
      options: Box::new(options),
    });
  }

  /// Replaces what a viewport draws over its frame.
  pub fn set_overlays(&self, id: RenderViewportId, overlays: Vec<RenderOverlay>) {
    self.send(RenderCommand::Overlays { id, overlays });
  }

  /// Marks what of a viewport's level is selected, or nothing for `None`.
  pub fn set_selection(&self, id: RenderViewportId, selection: Option<RenderSelection>) {
    self.send(RenderCommand::Selection { id, selection });
  }

  /// Applies settings to every viewport, now and in every thread started later.
  pub fn configure(&self, settings: RenderSettings) {
    self.lock().settings = settings;
    self.send(RenderCommand::Settings { settings });
  }

  /// Hands a command to the running thread, starting one for an attach; any other command has nothing to act on
  /// without a thread, since a stopped thread had no viewport left.
  fn send(&self, command: RenderCommand) {
    let mut link: MutexGuard<'_, RenderLink> = self.lock();
    let is_attach: bool = matches!(command, RenderCommand::Attach { .. });

    let command: RenderCommand = match &link.sender {
      Some(sender) => match sender.send(command) {
        Ok(()) => return,
        // The thread died: its viewports with it.
        Err(error) => error.0,
      },
      None => command,
    };

    link.sender = None;

    if is_attach {
      // A thread that cannot start leaves the viewport undrawn; the next attach tries again.
      let Some(sender) = self.spawn(&mut link) else {
        return;
      };

      if sender.send(command).is_err() {
        log::error!("The render thread stopped before its first viewport arrived");
      }
    }
  }

  fn spawn(&self, link: &mut RenderLink) -> Option<Sender<RenderCommand>> {
    let (sender, receiver) = channel();
    let shared: Arc<Mutex<RenderLink>> = Arc::clone(&self.link);
    let settings: RenderSettings = link.settings;
    let workers: RenderWorkers = self.workers.clone();
    let bundle: Arc<dyn RenderBundle> = Arc::clone(&self.bundle);
    let world: Arc<Mutex<dyn RenderWorld>> = Arc::clone(&self.world);

    if let Err(error) = std::thread::Builder::new()
      .name("xrf-render".into())
      .spawn(move || RenderThread::new(receiver, shared, settings, workers, (bundle, world)).run())
    {
      log::error!("The render thread could not be started: {error}");

      return None;
    }

    link.sender = Some(sender.clone());

    Some(sender)
  }

  fn lock(&self) -> MutexGuard<'_, RenderLink> {
    self.link.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }
}
