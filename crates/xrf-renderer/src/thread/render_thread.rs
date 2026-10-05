use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec4};
use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::{
  ExecutedGraph, FrameGraph, GraphBindings, GraphColorAttachment, GraphCompileOptions, GraphRuntime, GraphTexture,
  GraphTextureAccess,
};

use crate::camera::camera_view::CameraView;
use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_pick::RenderPick;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_surface_color::RenderSurfaceColor;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::frame_phases::FramePhases;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_input::RenderViewInput;
use crate::host::render_window_host::RenderWindowHost;
use crate::host::render_world::RenderWorld;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_view::SceneView;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::texture::sky_texture_requests::SkyTextureRequests;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu_state::GpuState;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_workers::RenderWorkers;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::pick_in_flight::PickInFlight;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::acquired_window::AcquiredWindow;
use crate::window::composed_view::ComposedView;
use crate::window::render_window::RenderWindow;
use crate::window::window_backdrop::WindowBackdrop;

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

/// How often a debug build looks for edited shader files.
const SHADER_RELOAD: Duration = Duration::from_millis(500);

/// A viewport's readied frame, with the pick it draws and what unprojects that pick's depth.
type ViewFrame = (RenderViewportId, LevelFrame, Option<(PendingPick, (Mat4, Vec2))>);

/// The render thread: owns the GPU and every viewport, and is the only place GPU work happens.
pub struct RenderThread {
  receiver: Receiver<RenderCommand>,
  link: Arc<Mutex<RenderLink>>,
  settings: RenderSettings,
  workers: RenderWorkers,
  /// The files the renderer ships with, which the GPU's passes read when it starts.
  bundle: Arc<dyn RenderBundle>,
  /// The world it draws, asked for each viewport's frame.
  world: Arc<Mutex<dyn RenderWorld>>,
  shaders: ShaderLibrary,
  gpu: Option<GpuState>,
  /// What the frames' graphs keep between frames, their passes' GPU timer among them; made with the GPU.
  runtime: Option<GraphRuntime>,
  /// Why the GPU last failed to start, and when.
  failure: Option<(String, Instant)>,
  /// The windows viewports are drawn into, by their key.
  hosts: HashMap<u64, Arc<dyn RenderWindowHost>>,
  viewports: BTreeMap<RenderViewportId, RenderViewport>,
  /// The scenes the world streams, each drawn by every viewport showing it; one the world let go is kept while a
  /// viewport still shows it.
  scenes: HashMap<RenderSceneId, LevelScene>,
  /// When the last viewport detached, while none is attached.
  idle_since: Option<Instant>,
  last_frame: Instant,
  /// When the next frame is due under a limited rate: the last one's due time and an interval, so frames keep step.
  frame_due: Option<Instant>,
  /// When each window last presented, which is what a frame's interval is measured from.
  last_present: HashMap<u64, Instant>,
  shaders_checked: Instant,
  /// When the textures no scene samples were last freed.
  textures_swept: Instant,
}

impl RenderThread {
  pub fn new(
    receiver: Receiver<RenderCommand>,
    link: Arc<Mutex<RenderLink>>,
    settings: RenderSettings,
    workers: RenderWorkers,
    (bundle, world): (Arc<dyn RenderBundle>, Arc<Mutex<dyn RenderWorld>>),
  ) -> Self {
    let now: Instant = Instant::now();

    Self {
      receiver,
      link,
      settings,
      workers,
      bundle,
      world,
      shaders: ShaderLibrary::default(),
      gpu: None,
      runtime: None,
      failure: None,
      hosts: HashMap::new(),
      viewports: BTreeMap::new(),
      scenes: HashMap::new(),
      idle_since: Some(now),
      last_frame: now,
      frame_due: None,
      last_present: HashMap::new(),
      shaders_checked: now,
      textures_swept: now,
    }
  }

  pub fn run(mut self) {
    log::info!("Render thread started");

    loop {
      if self.is_drawing() {
        // A limited rate waits out what is left of a frame's interval, hearing commands meanwhile.
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
            .frame_due
            .map_or(Duration::ZERO, |due| due.saturating_duration_since(Instant::now()));

          if left.is_zero() {
            break;
          }

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
      RenderCommand::Settings { settings } => self.settings = settings,
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
      RenderCommand::Pick { id, x, y, reply } => {
        let pick: PendingPick = PendingPick { x, y, reply };

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
          .zip(self.get_asked_scene(id).and_then(|scene| self.scenes.get(&scene)))
          .map(|(gpu, scene)| scene.describe_load(&gpu.textures));

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

  /// Starts the GPU when there is none, or again after it was lost.
  fn ensure_gpu(&mut self) {
    if self.gpu.as_ref().is_some_and(|gpu| gpu.context.is_lost()) {
      log::warn!("Restarting the renderer after its GPU was lost");
      self.gpu = None;
      // Its pool, bind groups and timer belong to the device that was lost.
      self.runtime = None;

      // Its scenes went with it; the world's next posts go to scenes made again.
      self.scenes.clear();

      for viewport in self.viewports.values_mut() {
        viewport.binding = None;
        viewport.shown = None;
      }
    }

    if self.gpu.is_some() || self.failure.as_ref().is_some_and(|(_, when)| when.elapsed() < RETRY) {
      return;
    }

    match GpuContext::create(RenderBackend::from_environment())
      .and_then(|context| GpuState::new(context, &self.shaders, &self.workers, self.bundle.as_ref()))
    {
      Ok(gpu) => {
        self.runtime = Some(GraphRuntime::new(&gpu.context.device, &gpu.context.queue));
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

  fn frame(&mut self) {
    let now: Instant = Instant::now();
    let delta: f32 = now.duration_since(self.last_frame).as_secs_f32();

    self.last_frame = now;
    // Kept in step from the last due time, so a frame woken a little late does not push every later one; one drawn
    // later than its interval is followed by the next at once.
    self.frame_due = self
      .settings
      .frame_rate
      .get_interval()
      .map(|interval| (self.frame_due.unwrap_or(now) + interval).max(now));

    // The world runs inline before the frame: it moves each viewport's camera and weather, then streams each scene and
    // plays what lives in it, and answers all of it.
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

    let feedback: HashMap<RenderSceneId, RenderSceneFeedback> = self
      .scenes
      .iter_mut()
      .map(|(id, scene)| (*id, scene.take_feedback()))
      .collect();
    let scene_frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(feedback);

    drop(world);

    if cfg!(debug_assertions) && now.duration_since(self.shaders_checked) >= SHADER_RELOAD {
      self.shaders_checked = now;

      if self.shaders.reload()
        && let Some(gpu) = &mut self.gpu
      {
        gpu.refresh(&self.shaders);
      }
    }

    // The scenes take in what the world posted, whether or not any window can be drawn this frame, so no post is lost.
    let Some(gpu) = &mut self.gpu else {
      return;
    };
    let mut encoder: wgpu::CommandEncoder = gpu.context.device.create_command_encoder(&Default::default());

    let mut scene_frames: HashMap<RenderSceneId, RenderSceneFrame> = scene_frames;
    let live: HashSet<RenderSceneId> = scene_frames.keys().copied().collect();

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
        (&gpu.context.device, &gpu.context.queue, &mut encoder),
        (&mut gpu.textures, &mut gpu.weather_textures),
        &gpu.grass,
        frame,
      );
    }

    // Each viewport shows the scene it asks for: a level at once; a scene of models alone once it can be drawn whole,
    // the one before drawn until then, so a model swapped never leaves the viewport empty.
    for viewport in self.viewports.values_mut() {
      let asked: Option<RenderSceneId> = viewport.world.scene;
      let shown: Option<RenderSceneId> = viewport.shown.as_ref().map(|(id, _)| *id);

      if asked == shown {
        continue;
      }

      let Some(asked) = asked else {
        viewport.shown = None;

        continue;
      };
      let Some(scene) = self.scenes.get(&asked) else {
        continue;
      };

      if shown.is_none() || scene.source.get_sector_count() > 0 || scene.is_ready(&gpu.textures) {
        viewport.shown = Some((
          asked,
          SceneView::new(&gpu.context.device, &gpu.context.queue, &gpu.view_layout, scene),
        ));
      }
    }

    // A scene the world let go goes once no viewport shows it.
    let shown: HashSet<RenderSceneId> = self
      .viewports
      .values()
      .filter_map(|viewport| viewport.shown.as_ref().map(|(id, _)| *id))
      .collect();

    self.scenes.retain(|id, _| live.contains(id) || shown.contains(id));

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

    gpu.textures.update(&gpu.context.device, &gpu.context.queue);
    gpu.weather_textures.update(&gpu.context.device, &gpu.context.queue);

    self.draw_windows(now.elapsed(), encoder);

    let Some(gpu) = &self.gpu else {
      return;
    };
    // Delivers what earlier frames' reads asked for, without waiting on this frame's work.
    let _ = gpu.context.device.poll(wgpu::PollType::Poll);
    let (backend, adapter) = (gpu.context.backend.get_label(), gpu.context.adapter_name.as_str());

    for viewport in self.viewports.values_mut() {
      viewport.answer_readbacks(&self.scenes);
      viewport.report(
        now,
        (backend, adapter),
        (gpu.textures.get_bytes(), &mut self.scenes),
        self.runtime.as_mut().map(|runtime| &mut runtime.timer),
      );

      if let Some(scene) = viewport.shown.as_ref().and_then(|(id, _)| self.scenes.get(id)) {
        viewport.report_load(scene.describe_load(&gpu.textures));
      }
    }
  }

  /// Draws every window's viewports in one frame graph and presents each window: acquires every window's image first,
  /// skipping a window that cannot give one (minimised, occluded, outdated, lost) rather than waiting for it; readies
  /// each of their viewports' frames; declares the viewports' frames, then each window's composition and captures;
  /// submits once; and presents every window. `update` is what the frame spent before any window.
  fn draw_windows(&mut self, update: Duration, mut encoder: wgpu::CommandEncoder) {
    let Some(gpu) = &mut self.gpu else {
      return;
    };
    let is_vsync: bool = self.settings.frame_rate.is_vsync;
    let mut acquired: Vec<AcquiredWindow> = Vec::new();

    for (key, host) in &self.hosts {
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
      let acquiring: Instant = Instant::now();
      let Some((image, width, height)) = window.acquire(&gpu.context, is_vsync) else {
        // A lost surface is let go, so the next frame draws into a new one.
        if window.is_lost() {
          gpu.windows.remove(key);
        }

        continue;
      };
      let acquire: Duration = acquiring.elapsed();
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
            (&gpu.context.device, &gpu.context.queue),
            &self.shaders,
            format,
            (window.get_backdrop_slot(), &backdrop),
          )
          .inspect_err(|error| log::error!("The page's wash cannot be drawn: {error}"))
          .is_ok();

      acquired.push(AcquiredWindow {
        key: *key,
        image,
        format,
        drawn,
        clear: wgpu::Color {
          r: f64::from(backdrop.color[0]),
          g: f64::from(backdrop.color[1]),
          b: f64::from(backdrop.color[2]),
          a: 1.0,
        },
        is_washed,
        acquire,
      });
    }

    // With nothing to draw into, what the scenes took in still goes up.
    if acquired.is_empty() {
      gpu.context.queue.submit([encoder.finish()]);

      return;
    }

    let started: Instant = Instant::now();
    let mut phases: FramePhases = FramePhases {
      update,
      ..FramePhases::default()
    };
    // Each viewport's readied frame, with the pick it draws and what unprojects that pick's depth; its scene's driving
    // viewport first, which simulates the scene and declares its work, or the first drawn where that one is not.
    let mut frames: Vec<ViewFrame> = Vec::new();
    let mut drawn: Vec<(RenderViewportId, RenderRect, Option<RenderSceneId>)> = acquired
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

      if let Some(mut frame) = Self::ready_view(
        gpu,
        (viewport, scene),
        rect,
        &mut encoder,
        (&mut phases.load, &mut phases.prepare),
        is_first,
      ) {
        if let (true, Some(scene)) = (is_first, scene_id) {
          frame.1.is_scene_first = true;
          firsts.push((scene, *id));
        }

        frames.push(frame);
      }
    }

    // Each scene's light faces, as every view of it asked for them, readied from its first view's culling.
    for (scene_id, id) in &firsts {
      let (Some(scene), Some((_, view))) = (
        self.scenes.get_mut(scene_id),
        self.viewports.get(id).and_then(|viewport| viewport.shown.as_ref()),
      ) else {
        continue;
      };

      view.prepare_scene_shadows(
        scene,
        (&gpu.context.device, &gpu.context.queue),
        &mut encoder,
        &gpu.textures,
      );
    }

    let composing: Instant = Instant::now();

    if !frames.is_empty() {
      for format in acquired.iter().map(|window| window.format).collect::<HashSet<_>>() {
        if let Err(error) = gpu.present.prepare(&gpu.context.device, &self.shaders, format) {
          log::error!("Level viewport cannot be presented: {error}");
        }

        if let Err(error) = gpu.overlay.prepare(&gpu.context.device, &self.shaders, format) {
          log::error!("Level overlays cannot be drawn: {error}");
        }
      }
    }

    phases.compose = composing.elapsed();

    let Some(runtime) = self.runtime.as_mut() else {
      return;
    };
    let gpu: &GpuState = gpu;
    let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&gpu.context.device, &gpu.context.queue);
    // Copied out of their window's image after it is composed, before it is presented; read once each is back.
    let mut captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)> = Vec::new();

    for window in &acquired {
      for (id, _, shown) in &window.drawn {
        let Some(viewport) = self.viewports.get_mut(id) else {
          continue;
        };

        for reply in std::mem::take(&mut viewport.captures) {
          match FrameCapture::new(device, window.format, *shown, viewport.frame) {
            Ok(capture) => captures.push((*id, capture, reply)),
            Err(error) => {
              let _ = reply.send(Err(error));
            }
          }
        }
      }
    }

    runtime.timer.set_enabled(self.settings.is_gpu_timed);

    let recording: Instant = Instant::now();
    let targets: Vec<wgpu::TextureView> = acquired
      .iter()
      .map(|window| window.image.texture.create_view(&Default::default()))
      .collect();
    let executed: XrfResult<ExecutedGraph> = {
      let mut graph: FrameGraph<'_> = FrameGraph::new();
      let mut bindings: GraphBindings<'_> = GraphBindings::new();

      // Each viewport's frame, its passes timed as its own.
      for (id, frame, _) in &frames {
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
        view.record(
          scene,
          (&mut graph, &mut bindings, &mut *runtime),
          gpu.get_level_passes(),
          binding,
          &gpu.textures,
          frame,
        );
      }

      // Then each window: the page's backdrop, each viewport's picture or its grid cropped to what the window shows,
      // its overlays, and the captures copied out of it.
      graph.begin_owner(FrameGraph::FRAME_OWNER);

      for (window, target) in acquired.iter().zip(&targets) {
        let Some(grid) = gpu.get_grid(window.format) else {
          continue;
        };
        let composed: Vec<ComposedView<'_>> = window
          .drawn
          .iter()
          .filter_map(|(id, rect, shown)| {
            let viewport: &RenderViewport = self.viewports.get(id)?;
            let level: Option<&SceneView> = viewport.shown.as_ref().map(|(_, view)| view);

            Some(ComposedView {
              binding: viewport.binding.as_ref()?,
              present: level.and_then(SceneView::get_present_group),
              overlays: level.and_then(SceneView::get_overlays),
              rect: *rect,
              shown: *shown,
            })
          })
          .collect();
        let backdrop: Option<&WindowBackdrop> = gpu
          .windows
          .get(&window.key)
          .and_then(RenderWindow::get_backdrop)
          .filter(|_| window.is_washed);
        let (backdrop_pass, present, overlay) = (&gpu.backdrop, &gpu.present, &gpu.overlay);
        let format: wgpu::TextureFormat = window.format;

        graph.begin_group("window");

        let window_texture: GraphTexture = bindings.import_view(&mut graph, "window", target);

        graph
          .add_raster_pass("window")
          .color(GraphColorAttachment::new(
            window_texture,
            wgpu::LoadOp::Clear(window.clear),
          ))
          .record(move |context| {
            let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

            if let Some(backdrop) = backdrop {
              backdrop_pass.draw(pass, format, backdrop);
            }

            for view in &composed {
              let (rect, shown, binding): (RenderRect, RenderRect, &ViewBinding) =
                (view.rect, view.shown, view.binding);

              // The whole viewport, past the window's edges where it reaches them, cropped to what the window shows.
              pass.set_viewport(
                rect.x as f32,
                rect.y as f32,
                rect.width as f32,
                rect.height as f32,
                0.0,
                1.0,
              );
              pass.set_scissor_rect(shown.x as u32, shown.y as u32, shown.width, shown.height);

              match view.present {
                Some(group) => present.draw(pass, format, binding, group),
                None => grid.draw(pass, binding),
              }

              if let Some((group, overlays)) = view.overlays {
                overlay.draw(pass, format, (binding, group), overlays);
              }
            }
          });

        for (id, capture, _) in &captures {
          if !window.drawn.iter().any(|(drawn, ..)| drawn == id) {
            continue;
          }

          graph
            .add_encoder_pass("capture")
            .texture(window_texture, GraphTextureAccess::CopySource)
            .keep()
            .record(move |context| {
              let texture: &wgpu::Texture = context.get_texture(window_texture).texture;

              capture.encode(context.get_encoder(), texture);
            });
        }
      }

      graph
        .compile(&GraphCompileOptions::default())
        .and_then(|compiled| compiled.execute((device, queue), runtime, &bindings))
    };
    let executed: ExecutedGraph = match executed {
      Ok(executed) => executed,
      Err(error) => {
        log::error!("The frame cannot be drawn: {error}");

        // Nothing of it was submitted: its picks wait for the next frame, and its captures are refused.
        for (id, _, pick) in frames {
          if let (Some(viewport), Some((pick, _))) = (self.viewports.get_mut(&id), pick) {
            viewport.picks.insert(0, pick);
          }
        }

        let message: String = error.to_string();

        for (_, _, reply) in captures {
          let _ = reply.send(Err(XrfError::new_invalid_error(message.clone())));
        }

        return;
      }
    };
    let finishing: Instant = Instant::now();
    let mut commands: Vec<wgpu::CommandBuffer> = vec![encoder.finish()];
    let encoded: Duration = finishing.elapsed() + executed.encode;

    phases.record += recording.elapsed().saturating_sub(encoded);
    phases.encode += encoded;
    commands.extend(executed.commands);

    let submitting: Instant = Instant::now();

    queue.submit(commands);
    phases.submit = submitting.elapsed();
    runtime.timer.request();

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

    let cpu: Duration = started.elapsed();

    // Each window presents, and its viewports note the frame with its own acquire, present and interval.
    for window in acquired {
      let presenting: Instant = Instant::now();

      queue.present(window.image);

      let presented: Instant = Instant::now();
      let window_phases: FramePhases = FramePhases {
        acquire: window.acquire,
        present: presented.duration_since(presenting),
        ..phases
      };
      let interval: Duration = self
        .last_present
        .insert(window.key, presented)
        .map_or(Duration::ZERO, |last| presented.duration_since(last));

      for (id, _, _) in &window.drawn {
        if let Some(viewport) = self.viewports.get_mut(id) {
          viewport.frame += 1;
          viewport.record_frame(interval, cpu, &window_phases);
        }
      }
    }
  }

  /// Readies one viewport's frame of the scene it shows: writes its camera, asks for the textures its weather draws
  /// with, steps the scene's particles where it simulates for the scene (`is_simulating`), prepares its frame, and
  /// readies its graph's frame with a pick when one waits; none where it shows no scene. `rect` is its whole rectangle;
  /// what loading and preparing take is added to `load` and `prepare`.
  fn ready_view(
    gpu: &mut GpuState,
    (viewport, scene): (&mut RenderViewport, Option<&mut LevelScene>),
    rect: &RenderRect,
    encoder: &mut wgpu::CommandEncoder,
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
    let switches: Vec4 = Vec4::new(
      f32::from(u8::from(options.mode.surface_color == RenderSurfaceColor::Textured)),
      options.mode.is_bumped as u32 as f32,
      options.features.hemi_strength,
      0.0,
    );
    let is_scened: bool = scene.is_some();

    // A level is drawn at a share of the viewport and upscaled to it; nothing else is drawn but at its size.
    let render_scale: RenderScale = if is_scened {
      options.output.upscaling.scale
    } else {
      RenderScale::Native
    };
    // The settings' resolution first, as a share of the viewport, then the render scale's share of that.
    let resolution: f32 = options
      .output
      .render_height
      .filter(|_| is_scened)
      .map_or(1.0, |height| (height as f32 / rect.height.max(1) as f32).min(1.0));
    let drawn_rect: RenderRect = RenderRect {
      width: render_scale.get_drawn(((rect.width as f32 * resolution).round() as u32).max(1)),
      height: render_scale.get_drawn(((rect.height as f32 * resolution).round() as u32).max(1)),
      ..*rect
    };
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
        queue,
        &CameraUniform::new(&view, drawn_rect, switches)
          .with_wireframe(options.mode.is_wireframe)
          .with_surface_color(options.mode.surface_color)
          .with_motion((unjittered, unjittered))
          .with_asset_view(&options, drawn_rect.height as f32 / rect.height.max(1) as f32),
      );

      return None;
    };
    // A temporal resolve's jitter moves every scene pass's samples, never the view its history is measured by.
    let jitter: Vec2 = view_state.next_jitter(&options, rect.width as f32 / drawn_rect.width.max(1) as f32);
    let drawn: CameraView = view.jittered(jitter, Vec2::new(drawn_rect.width as f32, drawn_rect.height as f32));
    let unjittered: Mat4 = view.get_view_projection();
    let motion: (Mat4, Mat4) = view_state.get_state_mut().next_motion(unjittered);
    let selected: Option<StaticSelection> = view_state.resolve_selection(scene, selection.as_ref()).cloned();

    binding.write(
      queue,
      &CameraUniform::new(&drawn, drawn_rect, switches)
        .with_selection(selected.as_ref())
        .with_wireframe(options.mode.is_wireframe)
        .with_surface_color(options.mode.surface_color)
        .with_motion(motion)
        .with_asset_view(&options, drawn_rect.height as f32 / rect.height.max(1) as f32),
    );

    // An asset viewer lights by its rig rather than a weather, and plays none.
    let asset_lighting: Option<RenderLighting> = options.asset.lighting.as_ref().map(RenderLighting::for_asset);
    let (lighting, weather) = match &asset_lighting {
      Some(lighting) => (lighting, None),
      None => (&world.lighting, world.weather.as_ref()),
    };
    let loading: Instant = Instant::now();

    view_state.set_overlays(scene, device, overlays, *overlays_version, selection.as_ref());
    view_state.request_textures(scene, &mut gpu.weather_textures, (lighting, weather), &options);
    *load += loading.elapsed();

    let preparing: Instant = Instant::now();

    // The scene's effects step once a frame, by the camera standing for the actor.
    if is_simulating {
      scene.particles.simulate(&view, &options);
    }

    view_state.prepare(
      scene,
      device,
      queue,
      encoder,
      gpu.get_level_passes(),
      &view,
      ((drawn_rect.width, drawn_rect.height), *rect),
      world.camera.field_of_view,
      &options,
      (lighting, weather),
      &gpu.weather_textures,
      world.clock_rate,
      (&gpu.view_layout, &gpu.textures),
    );
    *prepare += preparing.elapsed();

    // One pick a frame, drawn from this frame's culled clusters while a readback is free for it.
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
      (device, queue),
      pick.as_ref().map(|(_, _, camera)| (camera, &gpu.view_layout)),
    );

    match (readied, pick) {
      (Some(frame), Some((pick, ndc, _))) if frame.pick_slot.is_some() => {
        Some((id, frame, Some((pick, (view.get_view_projection().inverse(), ndc)))))
      }
      (readied, pick) => {
        // A pick no readback is free for waits for the next frame.
        if let Some((pick, ..)) = pick {
          picks.insert(0, pick);
        }

        readied.map(|frame| (id, frame, None))
      }
    }
  }
}
