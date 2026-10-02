use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};

use xrf_error::XrfResult;

use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_capture::RenderCapture;
use crate::contract::render_input_event::RenderInputEvent;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_window_host::RenderWindowHost;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_thread::RenderThread;
use crate::viewport::pending_pick::PendingPick;

/// The renderer as an application holds it: cheap to make, and touching no GPU until a viewport is attached.
///
/// Every call is a message to the render thread, started by the first attach and stopped a few seconds after the
/// last detach, so a tool that never draws never starts a GPU.
#[derive(Default)]
pub struct Renderer {
  link: Arc<Mutex<RenderLink>>,
  next_id: AtomicU32,
}

impl Renderer {
  pub fn new() -> Self {
    Self::default()
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

  pub fn send_input(&self, id: RenderViewportId, event: RenderInputEvent) {
    self.send(RenderCommand::Input { id, event });
  }

  pub fn set_camera(&self, id: RenderViewportId, camera: RenderCamera) {
    self.send(RenderCommand::Camera { id, camera });
  }

  pub fn command_camera(&self, id: RenderViewportId, command: RenderCameraCommand) {
    self.send(RenderCommand::CameraCommand { id, command });
  }

  /// What a viewport's level draws under a point, css pixels from its corner, answered after its next frame; the
  /// answer never comes for a viewport the renderer does not draw.
  pub fn pick(&self, id: RenderViewportId, x: f32, y: f32) -> Receiver<XrfResult<Option<RenderLevelHit>>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::Pick {
      id,
      pick: PendingPick { x, y, reply },
    });

    answer
  }

  /// What each shader table entry of a viewport's level draws across the sectors resident, answered at once.
  pub fn measure_surfaces(&self, id: RenderViewportId) -> Receiver<Vec<RenderSurfaceGeometry>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::MeasureSurfaces { id, reply });

    answer
  }

  /// What became of every texture a viewport's level samples, answered at once.
  pub fn describe_textures(&self, id: RenderViewportId) -> Receiver<Vec<RenderTextureReport>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::DescribeTextures { id, reply });

    answer
  }

  /// A viewport's next presented frame, read back; the answer never comes for a viewport the renderer does not draw.
  pub fn capture_viewport(&self, id: RenderViewportId) -> Receiver<XrfResult<RenderCapture>> {
    let (reply, answer) = channel();

    self.send(RenderCommand::Capture { id, reply });

    answer
  }

  pub fn set_view_options(&self, id: RenderViewportId, options: RenderViewOptions) {
    self.send(RenderCommand::Options { id, options });
  }

  /// Draws a level in a viewport, read from its source on the renderer's loader threads; `None` draws none.
  pub fn show_level(&self, id: RenderViewportId, source: Option<Arc<dyn RenderLevelSource>>) {
    self.send(RenderCommand::Level { id, source });
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
      let sender: Sender<RenderCommand> = self.spawn(&mut link);

      if sender.send(command).is_err() {
        log::error!("The render thread stopped before its first viewport arrived");
      }
    }
  }

  fn spawn(&self, link: &mut RenderLink) -> Sender<RenderCommand> {
    let (sender, receiver) = channel();
    let shared: Arc<Mutex<RenderLink>> = Arc::clone(&self.link);
    let settings: RenderSettings = link.settings;

    std::thread::Builder::new()
      .name("xrf-render".into())
      .spawn(move || RenderThread::new(receiver, shared, settings).run())
      .expect("The render thread could not be started");

    link.sender = Some(sender.clone());

    sender
  }

  fn lock(&self) -> MutexGuard<'_, RenderLink> {
    self.link.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }
}
