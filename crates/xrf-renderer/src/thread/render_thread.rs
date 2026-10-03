use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::capture_frame;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_window_host::RenderWindowHost;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_view::LevelView;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu_state::GpuState;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::render_window::RenderWindow;

/// How long a thread with nothing to draw waits for a command before looking again at its windows, which may have
/// been restored from minimised meanwhile.
const IDLE_WAIT: Duration = Duration::from_millis(50);

/// How long the thread keeps the GPU after its last viewport detaches, so a view shown again soon starts at once.
const IDLE_STOP: Duration = Duration::from_secs(5);

/// How long after a GPU failed to start it is tried again.
const RETRY: Duration = Duration::from_secs(2);

/// How often a debug build looks for edited shader files.
const SHADER_RELOAD: Duration = Duration::from_millis(500);

/// The render thread: owns the GPU and every viewport, and is the only place GPU work happens.
pub struct RenderThread {
  receiver: Receiver<RenderCommand>,
  link: Arc<Mutex<RenderLink>>,
  settings: RenderSettings,
  shaders: ShaderLibrary,
  gpu: Option<GpuState>,
  /// Why the GPU last failed to start, and when.
  failure: Option<(String, Instant)>,
  /// The windows viewports are drawn into, by their key.
  hosts: HashMap<u64, Arc<dyn RenderWindowHost>>,
  viewports: BTreeMap<RenderViewportId, RenderViewport>,
  /// When the last viewport detached, while none is attached.
  idle_since: Option<Instant>,
  last_frame: Instant,
  /// When each window last presented, which is what a frame's interval is measured from.
  last_present: HashMap<u64, Instant>,
  shaders_checked: Instant,
}

impl RenderThread {
  pub fn new(receiver: Receiver<RenderCommand>, link: Arc<Mutex<RenderLink>>, settings: RenderSettings) -> Self {
    let now: Instant = Instant::now();

    Self {
      receiver,
      link,
      settings,
      shaders: ShaderLibrary::default(),
      gpu: None,
      failure: None,
      hosts: HashMap::new(),
      viewports: BTreeMap::new(),
      idle_since: Some(now),
      last_frame: now,
      last_present: HashMap::new(),
      shaders_checked: now,
    }
  }

  pub fn run(mut self) {
    log::info!("Render thread started");

    loop {
      if self.is_drawing() {
        while let Ok(command) = self.receiver.try_recv() {
          self.handle(command);
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
      RenderCommand::Input { id, event } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.camera.input(&event);
        }
      }
      RenderCommand::Camera { id, camera } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.camera.describe(camera);
        }
      }
      RenderCommand::CameraCommand { id, command } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.camera.command(command);
        }
      }
      RenderCommand::Settings { settings } => self.settings = settings,
      RenderCommand::Options { id, options } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.options = options;
        }
      }
      RenderCommand::Overlays { id, overlays } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.overlays = overlays;
          viewport.overlays_version += 1;
        }
      }
      RenderCommand::Pick { id, pick } => match self.viewports.get_mut(&id) {
        Some(viewport) if viewport.level.is_some() => viewport.picks.push(pick),
        // Nothing is drawn there to pick.
        _ => {
          let _ = pick.reply.send(Ok(None));
        }
      },
      RenderCommand::MeasureSurfaces { id, reply } => {
        let measured: Vec<RenderSurfaceGeometry> = self
          .viewports
          .get(&id)
          .and_then(|viewport| viewport.level_view.as_ref())
          .map(|level| level.measure_surfaces())
          .unwrap_or_default();

        let _ = reply.send(measured);
      }
      RenderCommand::DescribeTextures { id, reply } => {
        let described: Vec<RenderTextureReport> = self
          .gpu
          .as_ref()
          .zip(
            self
              .viewports
              .get(&id)
              .and_then(|viewport| viewport.level_view.as_ref()),
          )
          .map(|(gpu, level)| level.describe_textures(&gpu.textures))
          .unwrap_or_default();

        let _ = reply.send(described);
      }
      RenderCommand::Capture { id, reply } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.captures.push(reply);
        }
      }
      RenderCommand::Level { id, source } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.weather.show(source.clone());
          viewport.level = source;
          viewport.level_view = None;
        }
      }
      RenderCommand::Weather { id, play, transition } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.weather.play(play, transition);
        }
      }
      RenderCommand::WeatherControl { id, control } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.weather.set_control(control);
        }
      }
      RenderCommand::WeatherSeek { id, time } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.weather.seek(time);
        }
      }
      RenderCommand::WeatherEffect { id, name } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.weather.play_effect(name.as_deref());
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

  /// Starts the GPU when there is none, or again after it was lost.
  fn ensure_gpu(&mut self) {
    if self.gpu.as_ref().is_some_and(|gpu| gpu.context.is_lost()) {
      log::warn!("Restarting the renderer after its GPU was lost");
      self.gpu = None;

      for viewport in self.viewports.values_mut() {
        viewport.binding = None;
        viewport.level_view = None;
      }
    }

    if self.gpu.is_some() || self.failure.as_ref().is_some_and(|(_, when)| when.elapsed() < RETRY) {
      return;
    }

    match GpuContext::create(RenderBackend::from_environment())
      .and_then(|context| GpuState::new(context, &self.shaders))
    {
      Ok(gpu) => {
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

    for viewport in self.viewports.values_mut() {
      let height: f32 = viewport.get_css_height();

      viewport.camera.update(delta, height);

      // The weather is weighed where the camera stands, in engine space; a fade waits for the skies it fades into.
      let position: Vec3 = viewport.camera.get_pose().position.into();
      let source: Option<Arc<dyn RenderAssetSource>> =
        viewport.level.clone().map(|level| level as Arc<dyn RenderAssetSource>);
      let is_clouded: bool = viewport.options.is_clouded;
      let is_thundering: bool = viewport.options.is_thundering && viewport.options.is_lit;
      let mut weather_textures: Option<&mut WeatherTextureCache> =
        self.gpu.as_mut().map(|gpu| &mut gpu.weather_textures);

      viewport.weather.advance(
        now,
        [position.x, position.y, -position.z],
        is_thundering,
        |lighting: &RenderLighting| match (weather_textures.as_deref_mut(), &source) {
          (Some(cache), Some(source)) => cache.request_sky(&lighting.sky, is_clouded, source),
          _ => true,
        },
      );

      // The keyframe the clock walks to next has its skies fetched before it is reached.
      if let (Some(cache), Some(source), Some(next)) =
        (weather_textures, &source, viewport.weather.get_player().get_next())
      {
        let keyframe = &next.descriptor;

        for (reference, kind) in [
          (keyframe.sky_texture.as_str(), WeatherTextureKind::Cube),
          (keyframe.sky_texture_env.as_str(), WeatherTextureKind::Cube),
          (keyframe.clouds_texture.as_str(), WeatherTextureKind::Flat),
        ] {
          if !reference.is_empty() && !keyframe.sky_texture.is_empty() {
            cache.request(reference, kind, source);
          }
        }
      }
    }

    if cfg!(debug_assertions) && now.duration_since(self.shaders_checked) >= SHADER_RELOAD {
      self.shaders_checked = now;

      if self.shaders.reload()
        && let Some(gpu) = &mut self.gpu
      {
        gpu.refresh(&self.shaders);
      }
    }

    if let Some(gpu) = &mut self.gpu {
      gpu.textures.update(&gpu.context.device, &gpu.context.queue);
      gpu.weather_textures.update(&gpu.context.device, &gpu.context.queue);
    }

    let windows: Vec<u64> = self.hosts.keys().copied().collect();

    for window in windows {
      self.draw_window(window);
    }

    let Some(gpu) = &self.gpu else {
      return;
    };
    // Delivers what earlier frames' reads asked for, without waiting on this frame's work.
    let _ = gpu.context.device.poll(wgpu::PollType::Poll);
    let (backend, adapter) = (gpu.context.backend.get_label(), gpu.context.adapter_name.as_str());

    for viewport in self.viewports.values_mut() {
      viewport.report(now, backend, adapter);
      viewport.publish_pose(now);
      viewport.publish_weather(now);

      if let Some(report) = viewport
        .level_view
        .as_mut()
        .and_then(|level| level.take_report(&gpu.textures))
      {
        viewport.report_load(report);
      }
    }
  }

  /// Draws every viewport of one window into its swapchain and presents it.
  fn draw_window(&mut self, window: u64) {
    let Some(gpu) = &mut self.gpu else {
      return;
    };
    let Some(host) = self.hosts.get(&window) else {
      return;
    };

    if !gpu.windows.contains_key(&window) {
      match RenderWindow::new(&gpu.context, Arc::clone(host)) {
        Ok(created) => {
          gpu.windows.insert(window, created);
        }
        Err(error) => {
          let message: String = error.to_string();

          for viewport in self.viewports.values_mut().filter(|it| it.window == window) {
            viewport.fail(&message);
          }

          return;
        }
      }
    }

    let presentation = self.settings.presentation;
    let Some((frame, width, height)) = gpu
      .windows
      .get_mut(&window)
      .and_then(|it| it.acquire(&gpu.context, presentation))
    else {
      return;
    };
    let started: Instant = Instant::now();
    let format: wgpu::TextureFormat = gpu.windows[&window].get_format();

    if let Err(error) = gpu.ensure_grid(&self.shaders, format) {
      log::error!("Viewport cannot be drawn: {error}");

      return;
    }

    let drawn: Vec<(RenderViewportId, RenderRect)> = self
      .viewports
      .values()
      .filter(|viewport| viewport.window == window)
      .filter_map(|viewport| viewport.get_drawn_rect(width, height).map(|rect| (viewport.id, rect)))
      .collect();
    // The page's background around the viewports, where a layout change has not reached the renderer yet.
    let clear: wgpu::Color = self
      .viewports
      .values()
      .find(|viewport| viewport.window == window)
      .and_then(|viewport| viewport.layout)
      .map(|layout| layout.clear.to_clear())
      .unwrap_or(wgpu::Color::BLACK);

    let mut encoder: wgpu::CommandEncoder = gpu.context.device.create_command_encoder(&Default::default());
    let mut is_lit: bool = false;
    let mut picked: Vec<(RenderViewportId, PendingPick, Mat4, Vec2)> = Vec::new();

    for (id, rect) in &drawn {
      let Some(viewport) = self.viewports.get_mut(id) else {
        continue;
      };
      let device: &wgpu::Device = &gpu.context.device;
      let queue: &wgpu::Queue = &gpu.context.queue;
      let scale: f32 = viewport.get_scale();
      let binding: &ViewBinding = viewport
        .binding
        .get_or_insert_with(|| ViewBinding::new(device, &gpu.view_layout));
      // A lit and fogged level ends where its fog is total, or at the weather's far plane, as the engine's does.
      let far_limit: f32 = viewport
        .weather
        .get_lighting()
        .fog
        .filter(|_| viewport.options.is_lit && viewport.options.is_fogged)
        .map_or(f32::INFINITY, |fog| fog.get_total_distance());
      let view: CameraView = viewport
        .camera
        .get_view(rect.width as f32 / rect.height as f32, far_limit);

      let options: RenderViewOptions = viewport.options.clone();
      // How far the water moves what is seen through it, none where it does not distort.
      let water = options.water;
      let distortion: f32 = if water.is_enabled && water.is_distorted && options.is_lit {
        water.distortion
      } else {
        0.0
      };
      let switches: Vec4 = Vec4::new(
        options.is_textured as u32 as f32,
        options.is_bumped as u32 as f32,
        options.hemi_strength,
        distortion,
      );

      // A level is drawn at a share of the viewport and upscaled to it; nothing else is drawn but at its size.
      let render_scale: RenderScale = if viewport.level.is_some() {
        options.upscaling.scale
      } else {
        RenderScale::Native
      };
      // The settings' resolution first, as a share of the viewport, then the render scale's share of that.
      let resolution: f32 = options
        .render_height
        .filter(|_| viewport.level.is_some())
        .map_or(1.0, |height| (height as f32 / rect.height.max(1) as f32).min(1.0));
      let drawn_rect: RenderRect = RenderRect {
        width: render_scale.get_drawn(((rect.width as f32 * resolution).round() as u32).max(1)),
        height: render_scale.get_drawn(((rect.height as f32 * resolution).round() as u32).max(1)),
        ..*rect
      };
      // A temporal resolve's jitter moves every scene pass's samples, never the view its history is measured by.
      let jitter: Vec2 = viewport.level_view.as_mut().map_or(Vec2::ZERO, |level| {
        level.next_jitter(&options, rect.width as f32 / drawn_rect.width.max(1) as f32)
      });
      let drawn: CameraView = view.jittered(jitter, Vec2::new(drawn_rect.width as f32, drawn_rect.height as f32));

      binding.write(
        queue,
        &CameraUniform::new(&drawn, drawn_rect, switches).with_wireframe(options.is_wireframe),
      );

      let Some(source) = &viewport.level else {
        continue;
      };
      let level: &mut LevelView = viewport
        .level_view
        .get_or_insert_with(|| LevelView::new(device, queue, &gpu.view_layout, Arc::clone(source)));

      level.set_timed(self.settings.is_gpu_timed);
      level.set_overlays(device, &viewport.overlays, viewport.overlays_version);
      level.load(
        device,
        queue,
        &mut encoder,
        (&mut gpu.textures, &mut gpu.weather_textures),
        &gpu.grass,
        (viewport.weather.get_lighting(), viewport.weather.get_level()),
        &options,
      );
      level.prepare(
        device,
        queue,
        &mut encoder,
        gpu.get_level_passes(),
        &view,
        ((drawn_rect.width, drawn_rect.height), *rect),
        viewport.camera.get_field_of_view(),
        &options,
        (viewport.weather.get_lighting(), viewport.weather.get_level()),
        &gpu.weather_textures,
      );
      level.record(
        device,
        queue,
        &mut encoder,
        gpu.get_level_passes(),
        &gpu.view_layout,
        binding,
        &gpu.textures,
      );
      is_lit = true;

      // One pick a frame, drawn from this frame's culled clusters.
      if !viewport.picks.is_empty() {
        let pick: PendingPick = viewport.picks.remove(0);
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

        level.record_pick(
          device,
          queue,
          &mut encoder,
          &gpu.static_gbuffer,
          &gpu.view_layout,
          &gpu.textures,
          &CameraUniform::new(&narrowed, pixel, switches),
        );
        picked.push((*id, pick, view.get_view_projection().inverse(), ndc));
      }
    }

    if is_lit && let Err(error) = gpu.present.prepare(&gpu.context.device, &self.shaders, format) {
      log::error!("Level viewport cannot be presented: {error}");
    }

    if is_lit && let Err(error) = gpu.overlay.prepare(&gpu.context.device, &self.shaders, format) {
      log::error!("Level overlays cannot be drawn: {error}");
    }

    let Some(grid) = gpu.get_grid(format) else {
      return;
    };
    let target: wgpu::TextureView = frame.texture.create_view(&Default::default());

    {
      let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("window"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(clear),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });

      for (id, rect) in &drawn {
        let Some(viewport) = self.viewports.get(id) else {
          continue;
        };
        let Some(binding) = viewport.binding.as_ref() else {
          continue;
        };

        pass.set_viewport(
          rect.x as f32,
          rect.y as f32,
          rect.width as f32,
          rect.height as f32,
          0.0,
          1.0,
        );
        pass.set_scissor_rect(rect.x as u32, rect.y as u32, rect.width, rect.height);

        match viewport.level_view.as_ref().and_then(|level| level.get_present_group()) {
          Some(present) => gpu.present.draw(&mut pass, format, binding, present),
          None => grid.draw(&mut pass, binding),
        }

        if let Some((group, overlays)) = viewport.level_view.as_ref().and_then(|level| level.get_overlays()) {
          gpu.overlay.draw(&mut pass, format, (binding, group), overlays);
        }
      }
    }

    gpu.context.queue.submit([encoder.finish()]);

    for (id, pick, inverse, ndc) in picked {
      let Some(level) = self.viewports.get(&id).and_then(|it| it.level_view.as_ref()) else {
        continue;
      };
      let unproject = |depth: f32| -> Vec3 { inverse.project_point3(ndc.extend(depth)) };

      let _ = pick.reply.send(level.resolve_pick(&gpu.context.device, unproject));
    }

    for (id, _) in &drawn {
      if let Some(level) = self.viewports.get(id).and_then(|it| it.level_view.as_ref()) {
        level.request_stats();
      }
    }

    let cpu: Duration = started.elapsed();

    // Read back before presenting, after which the frame is no longer the renderer's to copy.
    for (id, rect) in &drawn {
      if let Some(viewport) = self.viewports.get_mut(id) {
        for reply in viewport.captures.drain(..) {
          let _ = reply.send(capture_frame(
            &gpu.context.device,
            &gpu.context.queue,
            &frame.texture,
            *rect,
          ));
        }
      }
    }

    gpu.context.queue.present(frame);

    let presented: Instant = Instant::now();
    let interval: Duration = self
      .last_present
      .insert(window, presented)
      .map_or(Duration::ZERO, |last| presented.duration_since(last));

    for (id, _) in &drawn {
      if let Some(viewport) = self.viewports.get_mut(id) {
        viewport.record_frame(interval, cpu);
      }
    }
  }
}
