use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_dlss::Dlss;
use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::{
  ExecutedGraph, ExecutedGroup, FrameGraph, GraphBindings, GraphReport, GraphRuntime, GraphTaggedTime,
};

use wgpu::util::StagingBelt;

use crate::camera::camera_view::CameraView;
use crate::context::gpu_context::GpuContext;
use crate::context::gpu_requirements::can_trace;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_dlss_availability::RenderDlssAvailability;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_pick::RenderPick;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::frame_pacer::FramePacer;
use crate::frame::frame_phases::FramePhases;
use crate::frame::frame_span_timer::FrameSpanTimer;
use crate::frame::frame_statistics::FrameStatistics;
use crate::frame::gpu_memory::GpuMemory;
use crate::frame::periodic_budget::PeriodicBudget;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_input::RenderViewInput;
use crate::host::render_window_host::RenderWindowHost;
use crate::host::render_world::RenderWorld;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::scene_view::SceneView;
use crate::scene::level::view_prepare::ViewPrepare;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::texture::sky_texture_requests::SkyTextureRequests;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::shader::shader_watcher::ShaderWatcher;
use crate::thread::gpu_state::GpuState;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_workers::RenderWorkers;
use crate::thread::reopening_scenes::ReopeningScenes;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::pick_in_flight::PickInFlight;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::composed_view::ComposedView;
use crate::window::render_window::RenderWindow;
use crate::window::window_backdrop::WindowBackdrop;
use crate::window::window_frame::WindowFrame;

/// How long a thread with nothing to draw waits for a command before looking again at its windows, which may have
/// been restored from minimised meanwhile.
const IDLE_WAIT: Duration = Duration::from_millis(50);

/// The longest a limited rate sleeps between looks at its commands while it waits for the next frame. The sleep is
/// high resolution, where a channel's timeout rounds up to the system timer's tick of about 16 milliseconds.
const PACING_POLL: Duration = Duration::from_millis(1);

/// How long the thread keeps the GPU after its last viewport detaches, so a view shown again soon starts at once.
const IDLE_STOP: Duration = Duration::from_secs(5);

/// How long after a GPU failed to start it is tried again.
const RETRY: Duration = Duration::from_secs(2);

/// How often the textures no scene samples any more are freed.
const TEXTURE_SWEEP: Duration = Duration::from_secs(2);

/// How often the GPU allocator is asked what it holds, for the viewports' memory reports.
const ALLOCATOR_REPORT_EVERY: Duration = Duration::from_millis(500);

/// A viewport's readied frame, with the pick it draws and what unprojects that pick's depth.
type ViewFrame = (RenderViewportId, LevelFrame, Option<(PendingPick, (Mat4, Vec2))>);

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
  link: Arc<Mutex<RenderLink>>,
  settings: RenderSettings,
  workers: RenderWorkers,
  /// The files the renderer ships with, which the GPU's passes read when it starts.
  bundle: Arc<dyn RenderBundle>,
  /// Where the driver's compiled pipelines are kept between runs, none where nothing keeps them.
  store: Option<Arc<dyn RenderCacheStore>>,
  /// The world it draws, asked for each viewport's frame.
  world: Arc<Mutex<dyn RenderWorld>>,
  shaders: ShaderLibrary,
  gpu: Option<GpuState>,
  /// What the frames' graphs keep between frames, their passes' GPU timer among them; made with the GPU.
  runtime: Option<GraphRuntime>,
  /// How long the GPU spends on each frame, which the periodic work's budget is taken from; made with the GPU.
  frame_span: Option<FrameSpanTimer>,
  /// What the last described frame's graph came to, and when it was described: as often as the viewports report.
  graph_report: Option<(Instant, GraphReport)>,
  /// What the GPU allocator held when it was last asked: a report lists every allocation, so it is asked at most every
  /// `ALLOCATOR_REPORT_EVERY`.
  allocator_report: Option<(Instant, Option<GpuMemory>)>,
  /// The staging belt the frames' own writes go through, kept between frames; made with the GPU.
  belt: Option<StagingBelt>,
  /// What every view and scene draws once in so many frames, granted once a frame within the budget the frames'
  /// headroom leaves.
  periodic: PeriodicScheduler,
  periodic_budget: PeriodicBudget,
  /// Why the GPU last failed to start, and when.
  failure: Option<(String, Instant)>,
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
  /// When a limited rate's frames start and present: each presented at its due, the next started a lead ahead of it.
  pacer: FramePacer,
  /// When each window last called present, which is what a frame's interval is measured from.
  last_present: HashMap<u64, Instant>,
  /// Tells a debug build when a shader file was written, which it reads the shaders again for.
  shader_watcher: ShaderWatcher,
  /// When the textures no scene samples were last freed.
  textures_swept: Instant,
}

impl RenderThread {
  pub fn new(
    receiver: Receiver<RenderCommand>,
    link: Arc<Mutex<RenderLink>>,
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
      gpu: None,
      runtime: None,
      frame_span: None,
      graph_report: None,
      allocator_report: None,
      belt: None,
      periodic: PeriodicScheduler::new(),
      periodic_budget: PeriodicBudget::default(),
      failure: None,
      hosts: HashMap::new(),
      viewports: BTreeMap::new(),
      scenes: HashMap::new(),
      reopening: ReopeningScenes::default(),
      idle_since: Some(now),
      last_frame: now,
      pacer: FramePacer::default(),
      last_present: HashMap::new(),
      shader_watcher: ShaderWatcher::new(&ShaderLibrary::get_source_root()),
      textures_swept: now,
    }
  }

  /// Runs the thread until its queue goes: draws a frame while any viewport draws, paced to the settings' rate, and
  /// handles the commands sent between frames.
  pub fn run(mut self) {
    log::info!("Render thread started");

    loop {
      if self.is_drawing() {
        // A limited rate waits until the next frame starts, a lead ahead of its due, hearing commands meanwhile.
        loop {
          match self.receiver.try_recv() {
            Ok(command) => {
              self.handle(command);

              continue;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => return,
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
          Ok(command) => {
            self.handle(command);

            continue;
          }
          Err(RecvTimeoutError::Timeout) => {}
          Err(RecvTimeoutError::Disconnected) => break,
        }
      }

      self.forget_gone();

      if self.viewports.is_empty() {
        if self.idle_since.is_some_and(|since| since.elapsed() >= IDLE_STOP) && self.try_stop() {
          break;
        }

        continue;
      }

      self.ensure_gpu();
      self.frame();
    }

    log::info!("Render thread stopped");
  }

  /// Whether any viewport has somewhere to be drawn now: a frame is due each turn, rather than a wait.
  fn is_drawing(&self) -> bool {
    self.viewports.values().any(|viewport| {
      let host: Option<&Arc<dyn RenderWindowHost>> = self.hosts.get(&viewport.window);

      host.is_some_and(|host| {
        let (width, height) = host.get_client_size();

        !host.is_minimized() && viewport.get_drawn_rect(width, height).is_some()
      })
    })
  }

  fn handle(&mut self, command: RenderCommand) {
    match command {
      RenderCommand::Attach { id, host, sink } => {
        let window: u64 = host.get_key();

        self.hosts.entry(window).or_insert(host);
        self
          .viewports
          .insert(id, RenderViewport::new(id, window, sink, Instant::now()));
        self.idle_since = None;
      }
      RenderCommand::Detach { id } => {
        self.viewports.remove(&id);
        self.forget_unused_windows();
      }
      RenderCommand::Layout { id, layout } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.layout = Some(layout);
        }
      }
      RenderCommand::ReloadTextures { names } => {
        if let Some(gpu) = &mut self.gpu {
          gpu.textures.reload(&names);
          gpu.weather_textures.reload(&names);
        }
      }
      RenderCommand::Settings { settings } => {
        // A changed backend or GPU takes effect at once: a running GPU restarts unless it already matches the choice,
        // and a failed start retries without waiting. A change to the automatic GPU always restarts, since it may pick
        // a different one. A choice that cannot start falls back as at any start.
        let is_backend_changed: bool = settings.backend != self.settings.backend;
        let is_gpu_changed: bool = settings.gpu != self.settings.gpu;

        if is_backend_changed || is_gpu_changed {
          let preferred: RenderBackend = RenderBackend::get_preferred(RenderBackend::get_asked(settings.backend));
          let is_restarted: bool = self.gpu.as_ref().is_some_and(|gpu| {
            (is_backend_changed && gpu.context.backend != preferred)
              || (is_gpu_changed && settings.gpu.is_none_or(|asked| asked != gpu.context.gpu))
          });

          if is_restarted {
            log::info!("Restarting the renderer on {}", preferred.get_label());
            self.release_gpu();
          }

          self.failure = None;
        }

        self.settings = settings;
      }
      RenderCommand::Options { id, options } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.options = *options;
        }
      }
      RenderCommand::Overlays { id, overlays } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.overlays = overlays;
          viewport.overlays_version += 1;
        }
      }
      RenderCommand::Selection { id, selection } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.selection = selection;
        }
      }
      RenderCommand::Pick {
        id,
        x,
        y,
        is_through,
        reply,
      } => {
        let pick: PendingPick = PendingPick {
          x,
          y,
          is_through,
          reply,
        };

        match self.viewports.get_mut(&id) {
          Some(viewport) if viewport.shown.is_some() => viewport.picks.push(pick),
          // Nothing is drawn there to pick.
          Some(viewport) => {
            let _ = pick.reply.send(Ok(RenderPick {
              frame: viewport.frame,
              hit: None,
            }));
          }
          None => {
            let _ = pick.reply.send(Ok(RenderPick { frame: 0, hit: None }));
          }
        }
      }
      RenderCommand::DescribeTextures { id, reply } => {
        let described: Vec<RenderTextureReport> = self
          .gpu
          .as_ref()
          .zip(self.get_shown_scene(id))
          .map(|(gpu, scene)| scene.describe_textures(&gpu.textures))
          .unwrap_or_default();

        let _ = reply.send(described);
      }
      RenderCommand::DescribeLoad { id, reply } => {
        let described: Option<RenderLoadReport> = self
          .gpu
          .as_ref()
          .zip(
            self
              .get_asked_scene(id)
              .and_then(|asked| Some((asked, self.scenes.get(&asked)?))),
          )
          .map(|(gpu, (asked, scene))| scene.describe_load(asked, &gpu.textures));

        let _ = reply.send(described);
      }
      RenderCommand::DescribeFrame { id, reply } => {
        let described: Option<RenderFrameReport> = self
          .viewports
          .get(&id)
          .and_then(RenderViewport::get_frame_report)
          .cloned();

        let _ = reply.send(described);
      }
      RenderCommand::LocateSpawnObject { id, object, reply } => {
        let sphere: Option<[f32; 4]> = self
          .get_shown_scene(id)
          .and_then(|scene| scene.get_object_sphere(object))
          .map(|sphere| sphere.to_array());

        let _ = reply.send(sphere);
      }
      RenderCommand::Capture { id, reply } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.captures.push(reply);
        }
      }
    }
  }

  /// Detaches viewports whose page stopped listening.
  fn forget_gone(&mut self) {
    let before: usize = self.viewports.len();

    self.viewports.retain(|_, viewport| !viewport.is_gone());

    if self.viewports.len() != before {
      self.forget_unused_windows();
    }
  }

  /// Lets go of windows no viewport is drawn into, and starts the idle clock when none is left.
  fn forget_unused_windows(&mut self) {
    let used: Vec<u64> = self.viewports.values().map(|viewport| viewport.window).collect();

    self.hosts.retain(|key, _| used.contains(key));
    self.last_present.retain(|key, _| used.contains(key));

    if let Some(gpu) = &mut self.gpu {
      gpu.windows.retain(|key, _| used.contains(key));
    }

    if self.viewports.is_empty() && self.idle_since.is_none() {
      self.idle_since = Some(Instant::now());
    }
  }

  /// Stops the thread unless a command arrived meanwhile: checked and decided under the link's lock, which every
  /// command is sent under.
  fn try_stop(&mut self) -> bool {
    let link: Arc<Mutex<RenderLink>> = Arc::clone(&self.link);
    let mut link = link.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Ok(command) = self.receiver.try_recv() {
      drop(link);
      self.handle(command);

      return false;
    }

    link.sender = None;
    self.gpu = None;

    true
  }

  /// The scene a viewport asks to show now, as its world says it, ahead of the frame that catches up with it: so a poll
  /// after a switch never reads the scene before as the one asked for.
  fn get_asked_scene(&self, viewport: RenderViewportId) -> Option<RenderSceneId> {
    self
      .world
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .get_asked_scene(viewport)
  }

  /// The scene a viewport shows, none where it shows none.
  fn get_shown_scene(&self, viewport: RenderViewportId) -> Option<&LevelScene> {
    self.scenes.get(&self.viewports.get(&viewport)?.shown.as_ref()?.0)
  }

  /// Lets the GPU go, with everything made on its device, for `ensure_gpu` to start it again.
  fn release_gpu(&mut self) {
    // The device's work finished and the windows' swapchains let go before the device, so a backend started next finds
    // every window free of them; a lost device has no work to wait for.
    if let Some(mut gpu) = self.gpu.take() {
      if !gpu.context.is_lost() {
        let _ = gpu.context.device.poll(wgpu::PollType::wait_indefinitely());
      }

      gpu.windows.clear();
    }

    // Its pool, bind groups and timers belong to the device let go.
    self.runtime = None;
    self.frame_span = None;
    self.belt = None;

    // Its scenes went with it; the world streams each in again, from the start, into scenes made again, which the
    // viewports showing them wait for whole.
    self.scenes.clear();
    self.reopening.hold(
      self
        .viewports
        .values()
        .filter_map(|viewport| viewport.shown.as_ref().map(|(id, _)| *id)),
    );
    self
      .world
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .restart_scenes();

    for viewport in self.viewports.values_mut() {
      viewport.binding = None;
      viewport.show(None);
    }
  }

  /// Starts the GPU when there is none, or again after it was lost, on the backend asked for, else the first that
  /// starts.
  fn ensure_gpu(&mut self) {
    if self.gpu.as_ref().is_some_and(|gpu| gpu.context.is_lost()) {
      log::warn!("Restarting the renderer after its GPU was lost");
      self.release_gpu();
    }

    if self.gpu.is_some() || self.failure.as_ref().is_some_and(|(_, when)| when.elapsed() < RETRY) {
      return;
    }

    let dxc: Option<PathBuf> = self.bundle.locate_bundled(GpuContext::DXC_LIBRARY);
    let dlss: Option<PathBuf> = self
      .bundle
      .locate_bundled(&format!("{}/{}", GpuContext::DLSS_DIRECTORY, Dlss::RUNTIME_LIBRARY))
      .and_then(|library| library.parent().map(PathBuf::from));

    match GpuContext::create_preferred(
      RenderBackend::get_asked(self.settings.backend),
      self.settings.gpu,
      (dxc.as_deref(), dlss.as_deref()),
    )
    .and_then(|context| {
      let building: Instant = Instant::now();
      let built: XrfResult<GpuState> = GpuState::new(
        context,
        &self.shaders,
        &self.workers,
        (self.bundle.as_ref(), self.store.as_deref()),
      );

      log::info!("Renderer's passes built in {} ms", building.elapsed().as_millis());

      built
    }) {
      Ok(gpu) => {
        self.runtime = Some(GraphRuntime::new(&gpu.context.device, &gpu.context.queue));
        self.frame_span = Some(FrameSpanTimer::new(&gpu.context.device, &gpu.context.queue));
        self.gpu = Some(gpu);
        self.failure = None;

        for viewport in self.viewports.values_mut() {
          viewport.recover();
        }
      }
      Err(error) => {
        let message: String = error.to_string();

        log::error!("Renderer cannot start: {message}");

        for viewport in self.viewports.values_mut() {
          viewport.fail(&message);
        }

        self.failure = Some((message, Instant::now()));
      }
    }
  }

  /// One frame: the world advanced, the shaders reloaded where they changed, the scenes fed what the world posted
  /// (whether or not any window can be drawn, so no post is lost), each viewport shown the scene it asks for, the
  /// scenes no one needs let go, the textures kept, every window drawn, then the reports.
  fn frame(&mut self) {
    profiling::scope!("frame");

    let now: Instant = Instant::now();
    let delta: f32 = now.duration_since(self.last_frame).as_secs_f32();

    self.last_frame = now;
    self.pacer.begin(now, self.settings.frame_rate.get_interval());

    let scene_frames: HashMap<RenderSceneId, RenderSceneFrame> = self.advance_world(now, delta);

    if self.shader_watcher.take_changed()
      && self.shaders.reload()
      && let Some(gpu) = &mut self.gpu
    {
      gpu.refresh(&self.shaders);
    }

    let Some((mut encoder, span)) = self.begin_frame_work(delta) else {
      return;
    };
    let live: HashSet<RenderSceneId> = self.ingest_scenes(scene_frames, &mut encoder);

    self.show_asked_scenes();
    self.retain_scenes(&live);
    self.keep_textures(now);
    self.draw_windows((now.elapsed(), span), encoder);
    self.report_viewports(now);
  }

  /// Runs the world inline before the frame: it moves each viewport's camera and weather, then streams each scene and
  /// plays what lives in it, and answers what each scene takes in. Without a GPU no scene could take in what the world
  /// posts, so the scenes wait for one rather than stream into nothing.
  fn advance_world(&mut self, now: Instant, delta: f32) -> HashMap<RenderSceneId, RenderSceneFrame> {
    let mut world: MutexGuard<'_, dyn RenderWorld> = self.world.lock().unwrap_or_else(PoisonError::into_inner);

    for viewport in self.viewports.values_mut() {
      let mut skies: SkyTextureRequests<'_> = SkyTextureRequests {
        cache: self.gpu.as_mut().map(|gpu| &mut gpu.weather_textures),
        source: viewport
          .world
          .scene
          .and_then(|scene| self.scenes.get(&scene))
          .map(|scene| Arc::clone(&scene.source) as Arc<dyn RenderAssetSource>),
        is_clouded: viewport.options.show.is_clouded,
      };

      viewport.world = world.advance_view(
        viewport.id,
        RenderViewInput {
          now,
          delta,
          height: viewport.get_css_height(),
          options: &viewport.options,
          skies: &mut skies,
        },
      );
    }

    if self.gpu.is_none() {
      return HashMap::new();
    }

    let feedback: HashMap<RenderSceneId, RenderSceneFeedback> = self
      .scenes
      .iter_mut()
      .map(|(id, scene)| (*id, scene.take_feedback()))
      .collect();

    world.advance_scenes(feedback)
  }

  /// Opens the frame's encoder and grants its periodic work and streaming against one budget, the streaming's rounds
  /// first, from the timers read back since; answers the encoder and the GPU's milliseconds on the latest frame read
  /// back. None without a GPU.
  fn begin_frame_work(&mut self, delta: f32) -> Option<(FrameEncoder, Option<f32>)> {
    let gpu: &GpuState = self.gpu.as_ref()?;
    let belt: StagingBelt = self
      .belt
      .take()
      .unwrap_or_else(|| FrameEncoder::create_belt(&gpu.context.device));
    let mut encoder: FrameEncoder = FrameEncoder::new(&gpu.context.device, belt);
    let times: Vec<GraphTaggedTime> = self
      .runtime
      .as_mut()
      .map(|runtime| runtime.timer.take_tagged())
      .unwrap_or_default();
    let spans: Vec<(u64, f32)> = self.frame_span.as_mut().map(FrameSpanTimer::take).unwrap_or_default();
    let span: Option<f32> = spans.last().map(|(_, span)| *span);
    let budget: f32 = self.periodic_budget.update(
      (Duration::from_secs_f32(delta), span),
      self.settings.frame_rate,
      self.settings.periodic_budget,
    );

    self.periodic.begin(budget, (&times, &spans));

    if let Some(timer) = self.frame_span.as_mut() {
      timer.begin(&mut encoder, self.periodic.get_frame());
    }

    self.periodic.set_owner(u32::MAX);

    Some((encoder, span))
  }

  /// Feeds each scene what the world posted for it, making it where it is new; a scene keeps its acceleration
  /// structures only while a viewport traces rays in it (`RenderViewOptions::is_scene_traced`). Answers the scenes the
  /// world still holds.
  fn ingest_scenes(
    &mut self,
    mut scene_frames: HashMap<RenderSceneId, RenderSceneFrame>,
    encoder: &mut FrameEncoder,
  ) -> HashSet<RenderSceneId> {
    let live: HashSet<RenderSceneId> = scene_frames.keys().copied().collect();
    let traced: HashSet<RenderSceneId> = self
      .viewports
      .values()
      .filter(|viewport| viewport.options.is_scene_traced())
      .filter_map(|viewport| viewport.world.scene)
      .collect();
    let Some(gpu) = &mut self.gpu else {
      return live;
    };

    for (id, frame) in scene_frames.drain() {
      let scene: &mut LevelScene = self.scenes.entry(id).or_insert_with(|| {
        LevelScene::new(
          &gpu.context.device,
          &gpu.context.queue,
          &gpu.view_layout,
          Arc::clone(&frame.source),
          &self.workers,
        )
      });

      scene.advance(
        (&gpu.context.device, &gpu.context.queue, &mut *encoder),
        (&mut gpu.textures, &mut gpu.weather_textures),
        (&gpu.grass, &mut self.periodic),
        (frame, traced.contains(&id)),
      );
    }

    live
  }

  /// Shows each viewport the scene it asks for: a level at once, unless it is made again after a restart; a scene of
  /// models alone, or a level made again, once it can be drawn whole, the one before drawn until then, so a model
  /// swapped never leaves the viewport empty.
  fn show_asked_scenes(&mut self) {
    let Some(gpu) = &self.gpu else {
      return;
    };

    for viewport in self.viewports.values_mut() {
      let asked: Option<RenderSceneId> = viewport.world.scene;
      let shown: Option<RenderSceneId> = viewport.shown.as_ref().map(|(id, _)| *id);

      if asked == shown {
        continue;
      }

      let Some(asked) = asked else {
        viewport.show(None);

        continue;
      };
      let Some(scene) = self.scenes.get(&asked) else {
        continue;
      };
      let is_held: bool = self.reopening.is_held(asked);
      let is_shown: bool = match is_held {
        true => self.reopening.release(asked, scene, &gpu.textures),
        false => shown.is_none() || scene.source.get_sector_count() > 0 || scene.is_ready(&gpu.textures),
      };

      // A swapchain made right after another backend's let go of the window can present without the window showing it
      // until it is configured again; the restarted level is the frame that does.
      if is_held
        && is_shown
        && let Some(window) = gpu.windows.get(&viewport.window)
      {
        window.configure_again();
      }

      if is_shown {
        viewport.show(Some((
          asked,
          SceneView::new(&gpu.context.device, &gpu.context.queue, &gpu.view_layout, scene),
        )));
      }
    }
  }

  /// Lets a scene the world let go (`live` holds the ones it keeps) go once no viewport shows it.
  fn retain_scenes(&mut self, live: &HashSet<RenderSceneId>) {
    let shown: HashSet<RenderSceneId> = self
      .viewports
      .values()
      .filter_map(|viewport| viewport.shown.as_ref().map(|(id, _)| *id))
      .collect();

    self.scenes.retain(|id, _| live.contains(id) || shown.contains(id));
    self.reopening.retain(live);
  }

  /// Keeps the textures: every `TEXTURE_SWEEP` those no scene samples let go, then the uploads streamed and the
  /// weather's textures brought up.
  fn keep_textures(&mut self, now: Instant) {
    let Some(gpu) = &mut self.gpu else {
      return;
    };

    if now.duration_since(self.textures_swept) >= TEXTURE_SWEEP {
      self.textures_swept = now;

      let sampled: HashSet<u32> = self.scenes.values().flat_map(LevelScene::list_texture_slots).collect();
      let environments: HashSet<u32> = self
        .scenes
        .values()
        .flat_map(LevelScene::list_environment_slots)
        .collect();

      gpu.textures.retain(&sampled);
      gpu.textures.retain_environments(&environments);
    }

    let is_opening: bool = self.scenes.values().any(|scene| !scene.is_ready(&gpu.textures));

    gpu.textures.update(
      (&gpu.context.device, &gpu.context.queue),
      &mut self.periodic,
      is_opening,
    );
    gpu.weather_textures.update(&gpu.context.device, &gpu.context.queue);
  }

  /// Delivers what earlier frames' reads asked for, without waiting on this frame's work, and reports each viewport:
  /// its readbacks answered, its frame with the device, DLSS, the allocator's totals and the graph, and its scene's
  /// load.
  fn report_viewports(&mut self, now: Instant) {
    let Some(gpu) = &self.gpu else {
      return;
    };
    let _ = gpu.context.device.poll(wgpu::PollType::Poll);
    let dlss: [RenderDlssAvailability; 2] = match (Dlss::IS_BUILT, &gpu.context.dlss) {
      (false, _) => [RenderDlssAvailability::Unbuilt; 2],
      (true, Some(dlss)) => [
        RenderDlssAvailability::Available,
        if dlss.has_ray_reconstruction() {
          RenderDlssAvailability::Available
        } else {
          RenderDlssAvailability::Unsupported
        },
      ],
      (true, None) => [RenderDlssAvailability::Unsupported; 2],
    };
    let device: (RenderBackend, &str, bool, [RenderDlssAvailability; 2]) = (
      gpu.context.backend,
      gpu.context.adapter_name.as_str(),
      can_trace(&gpu.context.device),
      dlss,
    );

    if self
      .allocator_report
      .as_ref()
      .is_none_or(|(asked, _)| now.duration_since(*asked) >= ALLOCATOR_REPORT_EVERY)
    {
      let last: Option<&GpuMemory> = self.allocator_report.as_ref().and_then(|(_, held)| held.as_ref());
      let held: Option<GpuMemory> = GpuMemory::read((&gpu.context.adapter, &gpu.context.device), last);

      self.allocator_report = Some((now, held));
    }

    let allocator: Option<&GpuMemory> = self.allocator_report.as_ref().and_then(|(_, held)| held.as_ref());

    for viewport in self.viewports.values_mut() {
      viewport.answer_readbacks(&self.scenes);
      viewport.report(
        now,
        device,
        (allocator, &mut self.scenes),
        (
          self.runtime.as_mut().map(|runtime| &mut runtime.timer),
          self.graph_report.as_ref().map(|(_, report)| report),
        ),
      );

      if let Some((id, scene)) = viewport
        .shown
        .as_ref()
        .and_then(|(id, _)| Some((*id, self.scenes.get(id)?)))
      {
        viewport.report_load(scene.describe_load(id, &gpu.textures));
      }
    }
  }

  /// Draws every window's viewports in one frame graph and presents each window: acquires every window's image first,
  /// skipping a window that cannot give one (minimised, occluded, outdated, lost) rather than waiting for it; readies
  /// each of their viewports' frames; declares the viewports' frames, then each window's composition and captures;
  /// submits once; and presents every window. `update` is what the frame spent before any window.
  /// Readies each window this frame draws: its swapchain configured for its size now, made where it is new or was
  /// lost, the viewports laid out in it with the part each shows, and its page's backdrop; its image is acquired later.
  fn ready_windows(&mut self, encoder: &mut FrameEncoder) -> Vec<WindowFrame> {
    let Some(gpu) = &mut self.gpu else {
      return Vec::new();
    };
    let is_vsync: bool = self.settings.frame_rate.is_vsync;
    let mut windows: Vec<WindowFrame> = Vec::new();

    for (key, host) in &self.hosts {
      // A lost surface is let go, so this frame draws into a new one.
      if gpu.windows.get(key).is_some_and(RenderWindow::is_lost) {
        gpu.windows.remove(key);
      }

      if !gpu.windows.contains_key(key) {
        match RenderWindow::new(&gpu.context, Arc::clone(host)) {
          Ok(created) => {
            gpu.windows.insert(*key, created);
          }
          Err(error) => {
            let message: String = error.to_string();

            for viewport in self.viewports.values_mut().filter(|it| it.window == *key) {
              viewport.fail(&message);
            }

            continue;
          }
        }
      }

      let Some(format) = gpu.windows.get(key).map(RenderWindow::get_format) else {
        continue;
      };

      if let Err(error) = gpu.ensure_grid(&self.shaders, format) {
        log::error!("Viewport cannot be drawn: {error}");

        continue;
      }

      let Some(window) = gpu.windows.get_mut(key) else {
        continue;
      };
      let Some((width, height)) = window.ready(&gpu.context, is_vsync) else {
        continue;
      };
      // Each viewport's whole rectangle, which its camera and targets are sized by, and the part of it the window
      // shows, which its picture is cropped to.
      let drawn: Vec<(RenderViewportId, RenderRect, RenderRect)> = self
        .viewports
        .values()
        .filter(|viewport| viewport.window == *key)
        .filter_map(|viewport| {
          Some((
            viewport.id,
            viewport.layout?.rect,
            viewport.get_drawn_rect(width, height)?,
          ))
        })
        .collect();
      // The page's backdrop around the viewports, and where a layout change has not reached the renderer yet.
      let backdrop: BackdropUniform = self
        .viewports
        .values()
        .find(|viewport| viewport.window == *key)
        .and_then(|viewport| viewport.layout)
        .map(|layout| BackdropUniform::new(&layout.backdrop))
        .unwrap_or_default();

      let is_washed: bool = backdrop.is_washed()
        && gpu
          .backdrop
          .prepare(
            (&gpu.context.device, &mut *encoder),
            PipelineSource {
              cache: &gpu.pipelines,
              device: &gpu.context.device,
              shaders: &self.shaders,
            },
            format,
            (window.get_backdrop_slot(), &backdrop),
          )
          .inspect_err(|error| log::error!("The page's wash cannot be drawn: {error}"))
          .is_ok();

      windows.push(WindowFrame {
        key: *key,
        format,
        drawn,
        clear: wgpu::Color {
          r: f64::from(backdrop.color[0]),
          g: f64::from(backdrop.color[1]),
          b: f64::from(backdrop.color[2]),
          a: 1.0,
        },
        is_washed,
        image: None,
        acquire: Duration::ZERO,
      });
    }

    windows
  }

  /// Draws every window's viewports: `update` is what the frame spent before any window, `span` the GPU's milliseconds
  /// on the latest frame read back, where one came back. The GPU's state and the graph's runtime are taken out for the
  /// frame, so each phase is a method of the thread's own.
  fn draw_windows(&mut self, timing: (Duration, Option<f32>), mut encoder: FrameEncoder) {
    let windows: Vec<WindowFrame> = self.ready_windows(&mut encoder);

    self.answer_unseen_picks(&windows);

    let Some(mut gpu) = self.gpu.take() else {
      return;
    };

    // With nothing to draw into, what the scenes took in still goes up.
    if windows.is_empty() {
      let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

      gpu.context.queue.submit([commands]);
      self.belt = Some(belt);
    } else if let Some(mut runtime) = self.runtime.take() {
      self.draw_frame((&mut gpu, &mut runtime), timing, windows, encoder);
      self.runtime = Some(runtime);
    }

    self.gpu = Some(gpu);
  }

  /// Answers the picks of every viewport no window draws this frame, laid out nowhere or in a window with no image:
  /// there is nothing to pick in.
  fn answer_unseen_picks(&mut self, windows: &[WindowFrame]) {
    let seen: HashSet<RenderViewportId> = windows
      .iter()
      .flat_map(|window| window.drawn.iter().map(|(id, _, _)| *id))
      .collect();

    for viewport in self.viewports.values_mut() {
      if !seen.contains(&viewport.id) {
        viewport.answer_picks_unseen();
      }
    }
  }

  /// The frame proper, over windows to draw into: the views readied, the periodic work committed, the composition's
  /// pipelines readied, the captures taken, the graph recorded and executed, the frame submitted, each view's frame
  /// ended and each window presented.
  fn draw_frame(
    &mut self,
    (gpu, runtime): (&mut GpuState, &mut GraphRuntime),
    (update, span): (Duration, Option<f32>),
    mut windows: Vec<WindowFrame>,
    mut encoder: FrameEncoder,
  ) {
    let started: Instant = Instant::now();
    let mut phases: FramePhases = FramePhases {
      update,
      ..FramePhases::default()
    };
    let (frames, firsts): (Vec<ViewFrame>, Vec<(RenderSceneId, RenderViewportId)>) =
      self.ready_views(gpu, &windows, &mut encoder, &mut phases);

    self.commit_periodic(gpu, (&frames, &firsts), &mut encoder);

    let composing: Instant = Instant::now();

    if !frames.is_empty() {
      self.prepare_composition(gpu, &windows);
    }

    phases.compose = composing.elapsed();

    let gpu: &GpuState = gpu;
    let mut captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)> =
      self.take_captures(&gpu.context.device, &windows);

    runtime.timer.set_enabled(self.settings.is_gpu_timed);
    runtime.timer.set_tag(self.periodic.get_frame());

    let recording: Instant = Instant::now();
    let executed: ExecutedGraph = match self.record_frame((gpu, runtime), &mut windows, &frames, &mut captures, started)
    {
      Ok(executed) => executed,
      Err(error) => {
        self.abandon_frame(gpu, encoder, (frames, &firsts), captures, &error);

        return;
      }
    };
    let waited: Duration = windows.iter().map(|window| window.acquire).sum();
    let groups: Vec<ExecutedGroup> =
      self.submit_frame((gpu, runtime), executed, encoder, (recording, waited), &mut phases);

    self.end_view_frames(frames, captures);

    let cpu: Duration = started.elapsed().saturating_sub(waited);

    self.present_windows(&gpu.context.queue, windows, phases, (cpu, span), &groups);

    profiling::finish_frame!();
  }

  /// Readies each drawn viewport's frame, with the pick it draws and what unprojects that pick's depth: its scene's
  /// driving viewport first, which simulates the scene and declares its work, or the first drawn where that one is not.
  /// Answers the frames and each scene's first view, which draws the scene's own work.
  fn ready_views(
    &mut self,
    gpu: &mut GpuState,
    windows: &[WindowFrame],
    encoder: &mut FrameEncoder,
    phases: &mut FramePhases,
  ) -> (Vec<ViewFrame>, Vec<(RenderSceneId, RenderViewportId)>) {
    let mut frames: Vec<ViewFrame> = Vec::new();
    let mut drawn: Vec<(RenderViewportId, RenderRect, Option<RenderSceneId>)> = windows
      .iter()
      .flat_map(|window| window.drawn.iter())
      .map(|(id, rect, _)| {
        let scene: Option<RenderSceneId> = self
          .viewports
          .get(id)
          .and_then(|viewport| viewport.shown.as_ref().map(|(scene, _)| *scene));

        (*id, *rect, scene)
      })
      .collect();

    drawn.sort_by_key(|(id, _, scene)| {
      !scene
        .and_then(|scene| self.scenes.get(&scene))
        .is_some_and(|scene| scene.driver == Some(*id))
    });

    let mut firsts: Vec<(RenderSceneId, RenderViewportId)> = Vec::new();

    for (id, rect, scene_id) in &drawn {
      let scene_id: Option<RenderSceneId> = *scene_id;
      let Some(viewport) = self.viewports.get_mut(id) else {
        continue;
      };
      let is_first: bool = scene_id.is_some_and(|scene| firsts.iter().all(|(seen, _)| *seen != scene));
      let scene: Option<&mut LevelScene> = scene_id.and_then(|scene| self.scenes.get_mut(&scene));

      self.periodic.set_owner(id.0);

      match Self::ready_view(
        (gpu, &self.shaders),
        (viewport, scene),
        rect,
        (&mut *encoder, &mut self.periodic),
        (&mut phases.load, &mut phases.prepare),
        is_first,
      ) {
        Some(mut frame) => {
          if let (true, Some(scene)) = (is_first, scene_id) {
            frame.1.is_scene_first = true;
            firsts.push((scene, *id));
          }

          frames.push(frame);
        }
        // A view that draws nothing this frame draws none of what it offered.
        None => self.periodic.withdraw(id.0),
      }
    }

    (frames, firsts)
  }

  /// Grants the frame's periodic work: each scene's light faces offered as every view of it asked for them, then what
  /// was granted readied by each view, and each scene's light faces from its first view's culling.
  fn commit_periodic(
    &mut self,
    gpu: &GpuState,
    (frames, firsts): (&[ViewFrame], &[(RenderSceneId, RenderViewportId)]),
    encoder: &mut FrameEncoder,
  ) {
    for (scene_id, id) in firsts {
      if let Some(scene) = self.scenes.get_mut(scene_id) {
        self.periodic.set_owner(id.0);
        scene.lights.offer_shadows(&mut self.periodic);
      }
    }

    self.periodic.grant();

    for (id, ..) in frames {
      let Some((scene_id, view)) = self.viewports.get_mut(id).and_then(|viewport| viewport.shown.as_mut()) else {
        continue;
      };
      let Some(scene) = self.scenes.get(scene_id) else {
        continue;
      };

      view.commit_periodic(scene, &gpu.context.device, encoder, &self.periodic);
    }

    for (scene_id, id) in firsts {
      let (Some(scene), Some((_, view))) = (
        self.scenes.get_mut(scene_id),
        self.viewports.get(id).and_then(|viewport| viewport.shown.as_ref()),
      ) else {
        continue;
      };

      view.commit_scene_shadows(scene, &gpu.context.device, encoder, &self.periodic);
    }
  }

  /// Readies the present's and the overlays' pipelines for each window's image format.
  fn prepare_composition(&self, gpu: &mut GpuState, windows: &[WindowFrame]) {
    for format in windows.iter().map(|window| window.format).collect::<HashSet<_>>() {
      if let Err(error) = gpu.present.prepare(
        PipelineSource {
          cache: &gpu.pipelines,
          device: &gpu.context.device,
          shaders: &self.shaders,
        },
        format,
      ) {
        log::error!("Level viewport cannot be presented: {error}");
      }

      if let Err(error) = gpu.overlay.prepare(
        PipelineSource {
          cache: &gpu.pipelines,
          device: &gpu.context.device,
          shaders: &self.shaders,
        },
        format,
      ) {
        log::error!("Level overlays cannot be drawn: {error}");
      }
    }
  }

  /// The captures this frame copies out of their window's image once it is composed, before it is presented, each read
  /// once back. A viewport gives one capture a frame, the first asked, so captures asked together land on consecutive
  /// frames.
  fn take_captures(
    &mut self,
    device: &wgpu::Device,
    windows: &[WindowFrame],
  ) -> Vec<(RenderViewportId, FrameCapture, CaptureReply)> {
    let mut captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)> = Vec::new();

    for window in windows {
      for (id, _, shown) in &window.drawn {
        let Some(viewport) = self.viewports.get_mut(id) else {
          continue;
        };

        if !viewport.captures.is_empty() {
          let reply: CaptureReply = viewport.captures.remove(0);

          match FrameCapture::new(device, window.format, *shown, viewport.frame) {
            Ok(capture) => captures.push((*id, capture, reply)),
            Err(error) => {
              let _ = reply.send(Err(error));
            }
          }
        }
      }
    }

    captures
  }

  /// Records the frame's graph and executes it: each viewport's frame, its passes timed as its own; each window's
  /// image, acquired only once every view's work is recorded (the wait for it, while the GPU still draws the last
  /// frame, comes after the frame's own work on the render thread), a capture of a window that gave no image refused;
  /// then each window composed and the captures copied out of it.
  fn record_frame(
    &mut self,
    (gpu, runtime): (&GpuState, &mut GraphRuntime),
    windows: &mut [WindowFrame],
    frames: &[ViewFrame],
    captures: &mut Vec<(RenderViewportId, FrameCapture, CaptureReply)>,
    started: Instant,
  ) -> XrfResult<ExecutedGraph> {
    profiling::scope!("record");

    // Each window's image as the composition draws into it, once acquired.
    let mut targets: Vec<Option<wgpu::TextureView>> = Vec::with_capacity(windows.len());
    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let mut bindings: GraphBindings<'_> = GraphBindings::new();
    let mut outputs: HashMap<RenderViewportId, SceneOutput> = HashMap::with_capacity(frames.len());

    for (id, frame, _) in frames {
      let Some(viewport) = self.viewports.get(id) else {
        continue;
      };
      let (Some((scene_id, view)), Some(binding)) = (&viewport.shown, &viewport.binding) else {
        continue;
      };
      let Some(scene) = self.scenes.get(scene_id) else {
        continue;
      };

      graph.begin_owner(id.0);

      let output: Option<SceneOutput> = view.record(
        scene,
        (&mut graph, &mut bindings, &mut *runtime),
        gpu.get_level_passes(&self.shaders),
        binding,
        &gpu.textures,
        frame,
      );

      outputs.extend(output.map(|output| (*id, output)));
    }

    for window in windows.iter_mut() {
      if let Some(surface) = gpu.windows.get(&window.key) {
        window.acquire(surface);
      }

      targets.push(
        window
          .image
          .as_ref()
          .map(|image| image.texture.create_view(&Default::default())),
      );
    }
    let (kept, refused): (Vec<_>, Vec<_>) = std::mem::take(captures).into_iter().partition(|(id, ..)| {
      windows
        .iter()
        .any(|window| window.image.is_some() && window.drawn.iter().any(|(drawn, ..)| drawn == id))
    });

    *captures = kept;

    for (_, _, reply) in refused {
      let _ = reply.send(Err(XrfError::new_invalid_error(
        "The window gave no image to capture this frame",
      )));
    }

    graph.begin_owner(FrameGraph::FRAME_OWNER);

    for (window, target) in windows.iter().zip(&targets) {
      let (Some(target), Some(grid)) = (target, gpu.get_grid(window.format)) else {
        continue;
      };
      let composed: Vec<ComposedView<'_>> = window
        .drawn
        .iter()
        .filter_map(|(id, rect, shown)| Self::compose_view(&self.viewports, gpu, (*id, *rect, *shown), outputs.get(id)))
        .collect();
      let backdrop: Option<&WindowBackdrop> = gpu
        .windows
        .get(&window.key)
        .and_then(RenderWindow::get_backdrop)
        .filter(|_| window.is_washed);

      window.declare(
        (&mut graph, &mut bindings),
        (target, backdrop),
        composed,
        (&gpu.backdrop, &gpu.present, &gpu.overlay, grid),
        captures,
      );
    }

    let graph_report: &mut Option<(Instant, GraphReport)> = &mut self.graph_report;

    graph.compile(&self.settings.graph.to_options()).and_then(|compiled| {
      if graph_report
        .as_ref()
        .is_none_or(|(described, _)| started.duration_since(*described) >= FrameStatistics::REPORT_INTERVAL)
      {
        *graph_report = Some((started, compiled.describe()));
      }

      compiled.execute((&gpu.context.device, &gpu.context.queue), runtime, &bindings)
    })
  }

  /// A viewport as its window composes it: its picture from the frame it left (`output`) and its overlays over it, or
  /// its grid where it drew none; its whole rectangle and the part the window shows.
  fn compose_view<'a>(
    viewports: &'a BTreeMap<RenderViewportId, RenderViewport>,
    gpu: &'a GpuState,
    (id, rect, shown): (RenderViewportId, RenderRect, RenderRect),
    output: Option<&SceneOutput>,
  ) -> Option<ComposedView<'a>> {
    let viewport: &RenderViewport = viewports.get(&id)?;
    let level: Option<&SceneView> = viewport.shown.as_ref().map(|(_, view)| view);

    Some(ComposedView {
      binding: viewport.binding.as_ref()?,
      present: output.map(|output| PresentParameters {
        scene: output.scene,
        distortion: output.targets.distortion,
        depth_target: output.targets.depth,
        albedo_target: output.targets.albedo,
        normal_target: output.targets.normal,
        material_target: output.targets.material,
        light_target: output.targets.light,
        occlusion_target: output.targets.occlusion[0],
        present: output.present,
        upscaled: output.shown,
        motion_target: output.targets.motion,
        bloom_target: output.bloom,
        bloom_sampler: gpu.present.get_bloom_sampler(),
        indirect_light: output.indirect_light,
        reflections: output.reflections,
        reflection_ends: output.reflection_ends,
      }),
      overlays: output
        .zip(level.and_then(SceneView::get_overlays))
        .map(|(output, overlays)| {
          (
            OverlayParameters {
              depth_target: output.targets.depth,
              present: output.present,
              lighting: output.lighting,
            },
            overlays,
          )
        }),
      rect,
      shown,
    })
  }

  /// Gives up a frame its graph could not draw: what the frame's own encoder holds is not of the frame (the uploads and
  /// the acceleration structures the scenes built, which they count as done) and still goes up; what the graph copied
  /// out for reading back is dropped, its picks wait for the next frame, and its captures are refused.
  fn abandon_frame(
    &mut self,
    gpu: &GpuState,
    encoder: FrameEncoder,
    (frames, firsts): (Vec<ViewFrame>, &[(RenderSceneId, RenderViewportId)]),
    captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)>,
    error: &XrfError,
  ) {
    log::error!("The frame cannot be drawn: {error}");

    let scope: wgpu::ErrorScopeGuard = gpu.context.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

    gpu.context.queue.submit([commands]);
    self.belt = Some(belt);

    if let Some(error) = pollster::block_on(scope.pop()) {
      log::error!("The frame's uploads cannot be submitted: {error}");
    }

    for (id, _, pick) in frames {
      let Some(viewport) = self.viewports.get_mut(&id) else {
        continue;
      };

      if let Some((_, level)) = viewport.shown.as_mut() {
        level.abandon_frame();
      }

      if let Some((pick, _)) = pick {
        viewport.picks.insert(0, pick);
      }
    }

    for (scene_id, _) in firsts {
      if let Some(scene) = self.scenes.get_mut(scene_id) {
        scene.lights.abandon_shadows();
      }
    }

    let message: String = error.to_string();

    for (_, _, reply) in captures {
      let _ = reply.send(Err(XrfError::new_invalid_error(message.clone())));
    }
  }

  /// Submits the frame's own encoder, the graph's command buffers and the frame's span, timing the phases from
  /// `recording` on, the windows' acquire waits (`waited`) apart; asks for the timers back. Answers the graph's encode
  /// groups as executed.
  fn submit_frame(
    &mut self,
    (gpu, runtime): (&GpuState, &mut GraphRuntime),
    executed: ExecutedGraph,
    encoder: FrameEncoder,
    (recording, waited): (Instant, Duration),
    phases: &mut FramePhases,
  ) -> Vec<ExecutedGroup> {
    let device: &wgpu::Device = &gpu.context.device;
    let finishing: Instant = Instant::now();
    let mut commands: Vec<wgpu::CommandBuffer> = {
      profiling::scope!("finish", "frame");

      let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

      self.belt = Some(belt);

      vec![commands]
    };
    let encoded: Duration = finishing.elapsed() + executed.encode;

    phases.record += recording.elapsed().saturating_sub(encoded + waited);
    phases.encode += encoded;
    commands.extend(executed.commands);
    commands.extend(self.frame_span.as_mut().and_then(|timer| timer.end(device)));

    let submitting: Instant = Instant::now();

    {
      profiling::scope!("submit");

      gpu.context.queue.submit(commands);
    }
    phases.submit = submitting.elapsed();
    runtime.timer.request();

    if let Some(timer) = &self.frame_span {
      timer.request();
    }

    executed.groups
  }

  /// Ends each view's submitted frame: its history kept, its stats and its pick asked for back; and each capture asked
  /// for back.
  fn end_view_frames(&mut self, frames: Vec<ViewFrame>, captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)>) {
    for (id, frame, pick) in frames {
      let Some(viewport) = self.viewports.get_mut(&id) else {
        continue;
      };
      let Some((_, level)) = viewport.shown.as_mut() else {
        continue;
      };

      level.end_frame(&frame);
      level.request_stats();

      if let (Some(slot), Some((pick, unprojection))) = (frame.pick_slot, pick) {
        level.request_pick(slot);
        viewport.picks_in_flight.push(PickInFlight {
          pick,
          slot,
          unprojection,
          frame: viewport.frame,
        });
      }
    }

    for (id, capture, reply) in captures {
      capture.request();

      if let Some(viewport) = self.viewports.get_mut(&id) {
        viewport.captures_in_flight.push((capture, reply));
      }
    }
  }

  /// Presents each window, and notes the frame in its viewports with its own acquire, present and interval: `cpu` the
  /// frame's time on the render thread, the acquire waits apart, `span` the GPU's latest read back. A limited rate
  /// presents at the frame's due however long its work took, so frames are shown evenly; how long this one took from
  /// its start sets how far ahead of its due the next starts.
  fn present_windows(
    &mut self,
    queue: &wgpu::Queue,
    windows: Vec<WindowFrame>,
    phases: FramePhases,
    (cpu, span): (Duration, Option<f32>),
    groups: &[ExecutedGroup],
  ) {
    self.pacer.note_ready(self.last_frame.elapsed());
    self.pacer.wait_for_due();

    for window in windows {
      let Some(image) = window.image else {
        continue;
      };
      let presenting: Instant = Instant::now();

      {
        profiling::scope!("present");

        queue.present(image);
      }

      let presented: Instant = Instant::now();
      let window_phases: FramePhases = FramePhases {
        acquire: window.acquire,
        present: presented.duration_since(presenting),
        ..phases
      };
      // Between present calls, as PresentMon and the overlays built on it measure a frame, so how long a call blocks
      // does not read as uneven pacing.
      let interval: Duration = self
        .last_present
        .insert(window.key, presenting)
        .map_or(Duration::ZERO, |last| presenting.duration_since(last));

      for (id, _, _) in &window.drawn {
        if let Some(viewport) = self.viewports.get_mut(id) {
          viewport.frame += 1;
          viewport.record_frame((interval, cpu, span), &window_phases, groups);
        }
      }
    }
  }

  /// The sizes a viewport's frame is resolved at and drawn at over its rectangle `rect`: the settings' output
  /// resolution first, then the render scale's share of it, where it shows a scene (`is_scened`); nothing else is drawn
  /// but at the viewport's size.
  fn get_view_sizes(options: &RenderViewOptions, rect: &RenderRect, is_scened: bool) -> ((u32, u32), RenderRect) {
    let render_scale: RenderScale = if is_scened {
      options.output.upscaling.scale
    } else {
      RenderScale::Native
    };
    let resolved: (u32, u32) = if is_scened {
      options.output.get_output_size((rect.width, rect.height))
    } else {
      (rect.width, rect.height)
    };

    (
      resolved,
      RenderRect {
        width: render_scale.get_drawn(resolved.0),
        height: render_scale.get_drawn(resolved.1),
        ..*rect
      },
    )
  }

  /// Sets the textures' filtering and level of detail by a view's `options`, and asks for the heights where the
  /// enhanced parallax or the heights' debug view reads them.
  fn ask_textures(textures: &mut TextureCache, device: &wgpu::Device, options: &RenderViewOptions) {
    textures.set_anisotropy(device, options.features.texture_filtering.get_anisotropy());
    textures.set_lod(options.features.texture_lod);

    if options.features.parallax.is_enhanced() || options.mode.debug_view == RenderDebugView::Heights {
      textures.want_heights();
    }
  }

  /// Readies a view's graph frame, with one pick a frame drawn from this frame's culled clusters while a readback is
  /// free for it: the camera narrowed to the pick's texel of the viewport's rectangle (`rect`, at `scale`), its
  /// `switches` the view's. A pick no readback is free for waits for the next frame.
  fn ready_pick(
    (id, view_state, picks): (RenderViewportId, &mut SceneView, &mut Vec<PendingPick>),
    (device, encoder, view_layout): (&wgpu::Device, &mut FrameEncoder, &wgpu::BindGroupLayout),
    (view, rect, scale, switches): (&CameraView, &RenderRect, f32, Vec4),
  ) -> Option<ViewFrame> {
    let pick: Option<(PendingPick, Vec2, CameraUniform)> = (!picks.is_empty()).then(|| {
      let pick: PendingPick = picks.remove(0);
      let ndc: Vec2 = Vec2::new(
        (pick.x * scale + 0.5) / rect.width as f32 * 2.0 - 1.0,
        1.0 - (pick.y * scale + 0.5) / rect.height as f32 * 2.0,
      );
      let narrowed: CameraView = view.narrow_to(ndc, Vec2::new(2.0 / rect.width as f32, 2.0 / rect.height as f32));
      let pixel: RenderRect = RenderRect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
      };

      (pick, ndc, CameraUniform::new(&narrowed, pixel, switches))
    });
    let readied: Option<LevelFrame> = view_state.begin_frame(
      (device, encoder),
      pick
        .as_ref()
        .map(|(pick, _, camera)| (camera, view_layout, pick.is_through)),
    );

    match (readied, pick) {
      (Some(frame), Some((pick, ndc, _))) if frame.pick_slot.is_some() => {
        Some((id, frame, Some((pick, (view.get_view_projection().inverse(), ndc)))))
      }
      (readied, pick) => {
        if let Some((pick, ..)) = pick {
          picks.insert(0, pick);
        }

        readied.map(|frame| (id, frame, None))
      }
    }
  }

  /// Readies one viewport's frame of the scene it shows: writes its camera, asks for the textures its weather draws
  /// with, steps the scene's particles where it simulates for the scene (`is_simulating`), prepares its frame, and
  /// readies its graph's frame with a pick when one waits; none where it shows no scene. `rect` is its whole rectangle;
  /// what loading and preparing take is added to `load` and `prepare`.
  fn ready_view(
    (gpu, shaders): (&mut GpuState, &ShaderLibrary),
    (viewport, scene): (&mut RenderViewport, Option<&mut LevelScene>),
    rect: &RenderRect,
    (encoder, periodic): (&mut FrameEncoder, &mut PeriodicScheduler),
    (load, prepare): (&mut Duration, &mut Duration),
    is_simulating: bool,
  ) -> Option<ViewFrame> {
    let id: RenderViewportId = viewport.id;
    let device: &wgpu::Device = &gpu.context.device;
    let queue: &wgpu::Queue = &gpu.context.queue;
    let scale: f32 = viewport.get_scale();
    let binding: &ViewBinding = viewport
      .binding
      .get_or_insert_with(|| ViewBinding::new(device, &gpu.view_layout));
    // A lit and fogged level ends where its fog is total, or at the weather's far plane, as the engine's does.
    let far_limit: f32 = viewport
      .world
      .lighting
      .fog
      .filter(|_| viewport.options.mode.is_lit && viewport.options.show.is_fogged)
      .map_or(f32::INFINITY, |fog| fog.get_total_distance());
    let view: CameraView = viewport
      .world
      .camera
      .to_view(rect.width as f32 / rect.height as f32, far_limit);

    let options: RenderViewOptions = viewport.options.clone();
    let switches: Vec4 = CameraUniform::get_switches(&options);
    let (resolved, drawn_rect): ((u32, u32), RenderRect) = Self::get_view_sizes(&options, rect, scene.is_some());
    let RenderViewport {
      shown,
      world,
      selection,
      overlays,
      overlays_version,
      picks,
      ..
    } = viewport;
    let shown: Option<&mut SceneView> = shown.as_mut().map(|(_, view)| view);
    let (Some(scene), Some(view_state)) = (scene, shown) else {
      // Nothing to draw but its grid: its camera alone.
      let unjittered: Mat4 = view.get_view_projection();

      binding.write(
        encoder,
        &CameraUniform::new(&view, drawn_rect, switches)
          .with_wireframe(options.mode.is_wireframe)
          .with_surface_color(options.mode.surface_color)
          .with_motion((unjittered, unjittered))
          .with_asset_view(&options, drawn_rect.height as f32 / rect.height.max(1) as f32),
      );

      return None;
    };
    // A temporal resolve's jitter moves every scene pass's samples, never the view its history is measured by.
    let jitter: Vec2 = view_state.next_jitter(
      &options,
      resolved.0 as f32 / drawn_rect.width.max(1) as f32,
      gpu.context.dlss.is_some(),
    );
    let drawn: CameraView = view.jittered(jitter, Vec2::new(drawn_rect.width as f32, drawn_rect.height as f32));
    let unjittered: Mat4 = view.get_view_projection();
    let motion: (Mat4, Mat4) = view_state.next_motion(unjittered);
    let previous: Mat4 = view_state.next_jittered(drawn.get_view_projection());
    let selected: Option<StaticSelection> = view_state.resolve_selection(scene, selection.as_ref()).cloned();

    // An asset viewer lights by its rig rather than a weather, and plays none.
    let asset_lighting: Option<RenderLighting> = options.asset.lighting.as_ref().map(RenderLighting::for_asset);
    let (lighting, weather) = match &asset_lighting {
      Some(lighting) => (lighting, None),
      None => (&world.lighting, world.weather.as_ref()),
    };
    // The enhanced parallax shadows its relief towards the sun, as the view sees it.
    let to_sun: Vec3 = drawn.view.transform_vector3(-lighting.get_sun_direction());

    binding.write(
      encoder,
      &CameraUniform::new(&drawn, drawn_rect, switches)
        .with_parallax(&options.features.parallax, to_sun)
        .with_selection(selected.as_ref())
        .with_wireframe(options.mode.is_wireframe)
        .with_surface_color(options.mode.surface_color)
        .with_linear_scene(options.is_scene_linear())
        .with_motion(motion)
        .with_previous(previous)
        .with_asset_view(&options, drawn_rect.height as f32 / rect.height.max(1) as f32),
    );

    let loading: Instant = Instant::now();

    {
      profiling::scope!("load");

      view_state.set_overlays(scene, device, overlays, *overlays_version, selection.as_ref());
      view_state.request_textures(scene, &mut gpu.weather_textures, (lighting, weather), &options);
      Self::ask_textures(&mut gpu.textures, device, &options);
    }
    *load += loading.elapsed();

    let preparing: Instant = Instant::now();

    {
      profiling::scope!("prepare");

      // The scene's effects step once a frame, by the camera standing for the actor.
      if is_simulating {
        profiling::scope!("particles");

        scene.particles.simulate(&view, &options);
      }

      view_state.prepare(
        scene,
        encoder,
        &ViewPrepare {
          device,
          queue,
          dlss: gpu.context.dlss.as_ref(),
          passes: gpu.get_level_passes(shaders),
          view: &view,
          drawn: (drawn_rect.width, drawn_rect.height),
          resolved,
          output: *rect,
          field_of_view: world.camera.field_of_view,
          options: &options,
          lighting,
          weather,
          weather_textures: &gpu.weather_textures,
          weather_rate: world.clock_rate,
        },
      );
      view_state.offer_periodic(scene, device, periodic, &gpu.view_layout);
    }
    *prepare += preparing.elapsed();

    Self::ready_pick(
      (id, view_state, picks),
      (device, encoder, &gpu.view_layout),
      (&view, rect, scale, switches),
    )
  }
}
