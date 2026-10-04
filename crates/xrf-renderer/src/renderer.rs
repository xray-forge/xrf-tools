use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};

use xrf_error::XrfResult;

use crate::contract::render_camera::RenderCamera;
use crate::contract::render_camera_command::RenderCameraCommand;
use crate::contract::render_capture::RenderCapture;
use crate::contract::render_input_event::RenderInputEvent;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_level_problems::RenderLevelProblems;
use crate::contract::render_model_pose::RenderModelPose;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::contract::render_weather_control::RenderWeatherControl;
use crate::contract::render_weather_play::RenderWeatherPlay;
use crate::contract::render_weather_transition::RenderWeatherTransition;
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

  /// What a viewport's level could not draw the way it asked, answered at once.
  pub fn describe_problems(&self, id: RenderViewportId) -> Receiver<RenderLevelProblems> {
    let (reply, answer) = channel();

    self.send(RenderCommand::DescribeProblems { id, reply });

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

  /// Stands a viewport's skinned models in a pose, from its next frame on.
  pub fn pose_model(&self, id: RenderViewportId, pose: RenderModelPose) {
    self.send(RenderCommand::PoseModel { id, pose });
  }

  /// Replaces what a viewport draws over its frame.
  pub fn set_overlays(&self, id: RenderViewportId, overlays: Vec<RenderOverlay>) {
    self.send(RenderCommand::Overlays { id, overlays });
  }

  /// Marks what of a viewport's level is selected, or nothing for `None`.
  pub fn set_selection(&self, id: RenderViewportId, selection: Option<RenderSelection>) {
    self.send(RenderCommand::Selection { id, selection });
  }

  /// Draws a level in a viewport, read from its source on the renderer's loader threads; `None` draws none.
  pub fn show_level(&self, id: RenderViewportId, source: Option<Arc<dyn RenderLevelSource>>) {
    self.send(RenderCommand::Level { id, source });
  }

  /// Plays a weather in a viewport's level from now on, a cycle read from the level's source on the loader threads;
  /// one asked for before the level is shown plays once it is.
  pub fn play_weather(&self, id: RenderViewportId, play: RenderWeatherPlay, transition: RenderWeatherTransition) {
    self.send(RenderCommand::Weather { id, play, transition });
  }

  pub fn set_weather_control(&self, id: RenderViewportId, control: RenderWeatherControl) {
    self.send(RenderCommand::WeatherControl { id, control });
  }

  /// Plays a viewport's weather on from a time of day, in seconds since midnight, ending the effect playing.
  pub fn seek_weather(&self, id: RenderViewportId, time: f32) {
    self.send(RenderCommand::WeatherSeek { id, time });
  }

  /// Plays a weather effect over a viewport's cycle from its clock's time, or ends the one playing for `None`.
  pub fn play_weather_effect(&self, id: RenderViewportId, name: Option<String>) {
    self.send(RenderCommand::WeatherEffect { id, name });
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

    if let Err(error) = std::thread::Builder::new()
      .name("xrf-render".into())
      .spawn(move || RenderThread::new(receiver, shared, settings).run())
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
