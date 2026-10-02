use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_settings::RenderSettings;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_window_host::RenderWindowHost;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu_state::GpuState;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
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
      }
    }

    if self.gpu.is_some() || self.failure.as_ref().is_some_and(|(_, when)| when.elapsed() < RETRY) {
      return;
    }

    match GpuContext::create(RenderBackend::from_environment()) {
      Ok(context) => {
        self.gpu = Some(GpuState::new(context));
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
    }

    if cfg!(debug_assertions) && now.duration_since(self.shaders_checked) >= SHADER_RELOAD {
      self.shaders_checked = now;

      if self.shaders.reload()
        && let Some(gpu) = &mut self.gpu
      {
        gpu.refresh(&self.shaders);
      }
    }

    let windows: Vec<u64> = self.hosts.keys().copied().collect();

    for window in windows {
      self.draw_window(window);
    }

    let Some(gpu) = &self.gpu else {
      return;
    };
    let (backend, adapter) = (gpu.context.backend.get_label(), gpu.context.adapter_name.as_str());

    for viewport in self.viewports.values_mut() {
      viewport.report(now, backend, adapter);
      viewport.publish_pose(now);
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

    for (id, rect) in &drawn {
      let viewport: &mut RenderViewport = self.viewports.get_mut(id).expect("drawn viewport is attached");
      let binding: &ViewBinding = viewport
        .binding
        .get_or_insert_with(|| ViewBinding::new(&gpu.context.device, &gpu.view_layout));
      let aspect: f32 = rect.width as f32 / rect.height as f32;

      binding.write(
        &gpu.context.queue,
        &CameraUniform::new(&viewport.camera.get_view(aspect), rect.width, rect.height),
      );
    }

    let Some(grid) = gpu.get_grid(format) else {
      return;
    };
    let target: wgpu::TextureView = frame.texture.create_view(&Default::default());
    let mut encoder: wgpu::CommandEncoder = gpu.context.device.create_command_encoder(&Default::default());

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
        let Some(binding) = self.viewports.get(id).and_then(|it| it.binding.as_ref()) else {
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
        grid.draw(&mut pass, binding);
      }
    }

    gpu.context.queue.submit([encoder.finish()]);

    let cpu: Duration = started.elapsed();

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
