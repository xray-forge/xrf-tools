use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2};
use xrf_renderer_core::GraphReport;

use crate::capture::burst::burst_shot::BurstShot;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::frame_clock::FrameClock;
use crate::frame::frame_pacer::FramePacer;
use crate::frame::gpu_memory::GpuMemory;
#[cfg(debug_assertions)]
use crate::frame::jitter_guard::JitterGuard;
use crate::frame::periodic::periodic_budget::PeriodicBudget;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_window_host::RenderWindowHost;
use crate::host::render_world::RenderWorld;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_scene::LevelScene;
use crate::shader::shader_library::ShaderLibrary;
use crate::shader::shader_watcher::ShaderWatcher;
use crate::thread::frame_failure_log::FrameFailureLog;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_workers::RenderWorkers;
use crate::thread::reopening_scenes::ReopeningScenes;
use crate::thread::retry_backoff::RetryBackoff;
use crate::thread::running_gpu::RunningGpu;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::render_viewport::RenderViewport;

mod render_thread_capture;
mod render_thread_commands;
mod render_thread_draw;
mod render_thread_frame;
mod render_thread_lifecycle;
mod render_thread_submit;
mod render_thread_views;

/// How long a thread with nothing to draw waits for a command before looking again at its windows, which may have
/// been restored from minimised meanwhile.
const IDLE_WAIT: Duration = Duration::from_millis(50);

/// The longest a limited rate sleeps between looks at its commands while it waits for the next frame. The sleep is
/// high resolution, where a channel's timeout rounds up to the system timer's tick of about 16 milliseconds.
const PACING_POLL: Duration = Duration::from_millis(1);

/// How long the thread keeps the GPU after its last viewport detaches, so a view shown again soon starts at once.
const IDLE_STOP: Duration = Duration::from_secs(5);

/// How often the textures no scene samples any more are freed.
const TEXTURE_SWEEP: Duration = Duration::from_secs(2);

/// How often the GPU allocator is asked what it holds, for the viewports' and the application's memory reports.
const ALLOCATOR_REPORT_EVERY: Duration = Duration::from_millis(500);

/// A viewport's readied frame, with the pick it draws and what unprojects that pick's depth.
type ViewFrame = (RenderViewportId, LevelFrame, Option<(PendingPick, (Mat4, Vec2))>);

/// A capture or screenshot copied out of a viewport's frame, on its way back, and where its answer goes.
type CaptureCopy = (RenderViewportId, FrameCapture, CaptureReply);

/// A capture's shot copied out of a viewport's frame, on its way back.
type ShotCopy = (RenderViewportId, FrameCapture);

/// A burst's frame copied out of a viewport's frame, on its way back.
type BurstCopy = (RenderViewportId, BurstShot);

/// What the application hands the render thread beside its commands: the files it ships with, the world it draws, and
/// where it keeps the driver's compiled pipelines between runs.
pub(crate) type RenderHosts = (
  Arc<dyn RenderBundle>,
  Arc<Mutex<dyn RenderWorld>>,
  Option<Arc<dyn RenderCacheStore>>,
);

/// The render thread: owns the GPU and every viewport, and is the only place GPU work happens.
pub struct RenderThread {
  receiver: Receiver<RenderCommand>,
  /// What the queue shares with the thread, held weakly: once the last queue goes, so does its sender, and the thread
  /// stops as its channel disconnects.
  link: Weak<Mutex<RenderLink>>,
  settings: RenderSettings,
  workers: RenderWorkers,
  /// The files the renderer ships with, which the GPU's passes read when it starts.
  bundle: Arc<dyn RenderBundle>,
  /// Where the driver's compiled pipelines are kept between runs, none where nothing keeps them.
  store: Option<Arc<dyn RenderCacheStore>>,
  /// The world it draws, asked for each viewport's frame.
  world: Arc<Mutex<dyn RenderWorld>>,
  shaders: ShaderLibrary,
  /// The GPU and what the frames keep on it, while it runs.
  running: Option<RunningGpu>,
  /// What the last described frame's graph came to, and when it was described: as often as the viewports report.
  graph_report: Option<(Instant, GraphReport)>,
  /// What reads the jittered frame after a resolve in the graphs described, warned of once; a development check, kept
  /// out of the release builds.
  #[cfg(debug_assertions)]
  jitter_guard: JitterGuard,
  /// What the GPU allocator held when it was last asked: a report lists every allocation, so it is asked at most every
  /// `ALLOCATOR_REPORT_EVERY`.
  allocator_report: Option<(Instant, Option<GpuMemory>)>,
  /// What every view and scene draws once in so many frames, granted once a frame within the budget the frames'
  /// headroom leaves.
  periodic: PeriodicScheduler,
  periodic_budget: PeriodicBudget,
  /// Why the GPU last failed to start, and when it is tried again.
  failure: Option<RetryBackoff>,
  /// Why a window last could not be drawn into, and when it is tried again, by its key.
  window_failures: HashMap<u64, RetryBackoff>,
  /// The windows viewports are drawn into, by their key.
  hosts: HashMap<u64, Arc<dyn RenderWindowHost>>,
  viewports: BTreeMap<RenderViewportId, RenderViewport>,
  /// The scenes the world streams, each drawn by every viewport showing it; one the world let go is kept while a
  /// viewport still shows it.
  scenes: HashMap<RenderSceneId, LevelScene>,
  /// The scenes viewports showed when the GPU was let go, held back until they are whole again.
  reopening: ReopeningScenes,
  /// When the last viewport detached, while none is attached.
  idle_since: Option<Instant>,
  last_frame: Instant,
  /// The time every scene's frames are drawn at, which a tool may step; a burst paces its own while it draws. When the
  /// last frame was drawn on whichever paced it.
  clock: FrameClock,
  drawn_at: Instant,
  /// When a limited rate's frames start and present: each presented at its due, the next started a lead ahead of it.
  pacer: FramePacer,
  /// When each window last called present, which is what a frame's interval is measured from.
  last_present: HashMap<u64, Instant>,
  /// Tells a debug build when a shader file was written, which it reads the shaders again for.
  shader_watcher: ShaderWatcher,
  /// When the textures no scene samples were last freed.
  textures_swept: Instant,
  /// Who asked the thread to stop at once, answered once it stopped.
  stopping: Option<Sender<()>>,
  /// The composition's failures, which repeat every frame until a shader changes.
  composition_failures: FrameFailureLog,
}

impl RenderThread {
  pub fn new(
    receiver: Receiver<RenderCommand>,
    link: Weak<Mutex<RenderLink>>,
    settings: RenderSettings,
    workers: RenderWorkers,
    (bundle, world, store): RenderHosts,
  ) -> Self {
    let now: Instant = Instant::now();

    Self {
      receiver,
      link,
      settings,
      workers,
      bundle,
      world,
      store,
      shaders: ShaderLibrary::default(),
      running: None,
      graph_report: None,
      #[cfg(debug_assertions)]
      jitter_guard: JitterGuard::default(),
      allocator_report: None,
      periodic: PeriodicScheduler::new(),
      periodic_budget: PeriodicBudget::default(),
      failure: None,
      window_failures: HashMap::new(),
      hosts: HashMap::new(),
      viewports: BTreeMap::new(),
      scenes: HashMap::new(),
      reopening: ReopeningScenes::default(),
      idle_since: Some(now),
      last_frame: now,
      clock: FrameClock::default(),
      drawn_at: now,
      pacer: FramePacer::default(),
      last_present: HashMap::new(),
      shader_watcher: ShaderWatcher::new(&ShaderLibrary::get_source_root()),
      textures_swept: now,
      stopping: None,
      composition_failures: FrameFailureLog::default(),
    }
  }

  /// Runs the thread until it idles past `IDLE_STOP` or is asked to stop: draws a frame while any viewport draws, paced
  /// to the settings' rate, and handles the commands sent between frames.
  pub fn run(mut self) {
    log::info!("Render thread started");

    'running: loop {
      if self.is_drawing() {
        // A limited rate waits until the next frame starts, a lead ahead of its due, hearing commands meanwhile.
        loop {
          match self.receiver.try_recv() {
            Ok(command) => {
              self.handle(command);

              continue;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => break 'running,
          }

          let left: Duration = self
            .pacer
            .get_start(self.settings.frame_rate.get_interval())
            .map_or(Duration::ZERO, |start| start.saturating_duration_since(Instant::now()));

          if left.is_zero() {
            break;
          }

          // Slept: the start need not be exact, the lead's margin taking in a late wake, as the present's wait is.
          std::thread::sleep(left.min(PACING_POLL));
        }
      } else {
        match self.receiver.recv_timeout(IDLE_WAIT) {
          // A stop asked for is taken at once rather than after the next wait.
          Ok(command) => {
            self.handle(command);

            if self.stopping.is_none() {
              continue;
            }
          }
          Err(RecvTimeoutError::Timeout) => {}
          Err(RecvTimeoutError::Disconnected) => break 'running,
        }
      }

      self.forget_gone();

      if self.viewports.is_empty() {
        let is_due: bool = self.stopping.is_some() || self.idle_since.is_some_and(|since| since.elapsed() >= IDLE_STOP);

        if is_due && self.try_stop() {
          break 'running;
        }

        continue;
      }

      self.ensure_gpu();
      self.frame();
    }

    // Answered once everything the thread made on the GPU is let go: its scenes hold the device as long as the GPU
    // does.
    let reply: Option<Sender<()>> = self.stopping.take();

    drop(self);
    log::info!("Render thread stopped");

    if let Some(reply) = reply {
      let _ = reply.send(());
    }
  }

  /// Whether any viewport has somewhere to be drawn now, or captures, which draws into textures of its own whether or
  /// not its window shows anything: a frame is due each turn, rather than a wait.
  fn is_drawing(&self) -> bool {
    self.viewports.values().any(|viewport| {
      let host: Option<&Arc<dyn RenderWindowHost>> = self.hosts.get(&viewport.window);

      viewport.is_capturing()
        || host.is_some_and(|host| {
          let (width, height) = host.get_client_size();

          !host.is_hidden() && viewport.get_drawn_rect(width, height).is_some()
        })
    })
  }
}
