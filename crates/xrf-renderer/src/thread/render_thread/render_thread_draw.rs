use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::{
  ExecutedGraph, ExecutedGroup, FrameGraph, GraphBindings, GraphDeclaration, GraphOwner, GraphReport, GraphRuntime,
};

use wgpu::util::StagingBelt;

use crate::contract::render_rect::RenderRect;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::frame_phases::FramePhases;
use crate::frame::frame_statistics::FrameStatistics;
#[cfg(debug_assertions)]
use crate::frame::jitter_guard::JitterGuard;
use crate::frame::periodic::periodic_owner::PeriodicOwner;
use crate::host::render_scene_id::RenderSceneId;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::present_parameters::PresentParameters;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::scene_view::SceneView;
use crate::thread::gpu_state::GpuState;
use crate::thread::retry_backoff::RetryBackoff;
use crate::thread::running_gpu::RunningGpu;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::composed_view::ComposedView;
use crate::window::render_window::RenderWindow;
use crate::window::window_backdrop::WindowBackdrop;
use crate::window::window_frame::WindowFrame;

use super::{BurstCopy, CaptureCopy, RenderThread, ShotCopy, ViewFrame};

impl RenderThread {
  /// Readies each window this frame draws: its swapchain configured for its size now, made where it is new or was
  /// lost, the viewports laid out in it with the part each shows, and its page's backdrop; its image is acquired later.
  fn ready_windows(&mut self, encoder: &mut FrameEncoder) -> Vec<WindowFrame> {
    let Some(RunningGpu { gpu, .. }) = &mut self.running else {
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
        if self.window_failures.get(key).is_some_and(|failure| !failure.is_due()) {
          continue;
        }

        match RenderWindow::new(&gpu.context, Arc::clone(host)) {
          Ok(created) => {
            gpu.windows.insert(*key, created);
            self.window_failures.remove(key);
          }
          Err(error) => {
            let message: String = error.to_string();
            let is_new: bool = match self.window_failures.get_mut(key) {
              Some(failure) => failure.fail(message.clone()),
              None => {
                self.window_failures.insert(*key, RetryBackoff::new(message.clone()));

                true
              }
            };

            if is_new {
              log::error!("A window cannot be drawn into: {message}");
            }

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
      let size: Option<(u32, u32)> = window.ready(&gpu.context, is_vsync);
      // Each viewport's whole rectangle, which its camera and targets are sized by, and the part of it the window
      // shows, which its picture is cropped to. A capture draws into a texture of its own, so its viewport is drawn
      // however it is laid out and whether or not the window shows anything.
      let drawn: Vec<(RenderViewportId, RenderRect, RenderRect)> = self
        .viewports
        .values()
        .filter(|viewport| viewport.window == *key)
        .filter_map(|viewport| {
          let shown: Option<(RenderRect, RenderRect)> = size.and_then(|(width, height)| {
            Some((viewport.layout.as_ref()?.rect, viewport.get_drawn_rect(width, height)?))
          });

          match shown {
            Some((rect, clipped)) => Some((viewport.id, rect, clipped)),
            None if viewport.is_capturing() => Some((viewport.id, RenderRect::default(), RenderRect::default())),
            None => None,
          }
        })
        .collect();
      let Some((width, height)) = size else {
        if !drawn.is_empty() {
          windows.push(WindowFrame {
            key: *key,
            format,
            is_shown: false,
            drawn,
            clear: wgpu::Color::TRANSPARENT,
            is_backdrop_painted: false,
            image: None,
            acquire: Duration::ZERO,
          });
        }

        continue;
      };
      // The page's backdrop around the viewports, and where a layout change has not reached the renderer yet.
      let backdrop: BackdropUniform = self
        .viewports
        .values()
        .find(|viewport| viewport.window == *key)
        .and_then(|viewport| viewport.layout.as_ref())
        .map(|layout| BackdropUniform::new(&layout.backdrop))
        .unwrap_or_default();

      // A viewport over the whole window, as a full screen view is, hides the backdrop; the clear stands in for it.
      let window_rect: RenderRect = RenderRect {
        x: 0,
        y: 0,
        width,
        height,
      };
      let is_covered: bool = drawn.iter().any(|(_, _, shown)| *shown == window_rect);
      let is_backdrop_painted: bool = !is_covered
        && !backdrop.is_flat()
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
          .inspect_err(|error| {
            self
              .composition_failures
              .fail("The page's backdrop cannot be drawn", error)
          })
          .inspect(|()| self.composition_failures.succeed("The page's backdrop cannot be drawn"))
          .is_ok();

      windows.push(WindowFrame {
        key: *key,
        format,
        is_shown: true,
        drawn,
        clear: backdrop.get_clear(),
        is_backdrop_painted,
        image: None,
        acquire: Duration::ZERO,
      });
    }

    windows
  }

  /// Draws every window's viewports: `update` is what the frame spent before any window, `span` the GPU's milliseconds
  /// on the latest frame read back, where one came back. The running GPU is taken out for the frame, so each phase is a
  /// method of the thread's own.
  pub(super) fn draw_windows(&mut self, timing: (Duration, Option<f32>), mut encoder: FrameEncoder) {
    let windows: Vec<WindowFrame> = self.ready_windows(&mut encoder);

    self.answer_unseen_picks(&windows);

    let Some(mut running) = self.running.take() else {
      return;
    };

    // With nothing to draw into, what the scenes took in still goes up.
    if windows.is_empty() {
      let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

      running.gpu.context.queue.submit([commands]);
      running.belt = Some(belt);
    } else {
      self.draw_frame(&mut running, timing, windows, encoder);
    }

    self.running = Some(running);
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
  /// pipelines readied, the captures and bursts' frames taken, the graph recorded and executed, the frame submitted,
  /// each view's frame ended and each window presented.
  fn draw_frame(
    &mut self,
    running: &mut RunningGpu,
    (update, span): (Duration, Option<f32>),
    mut windows: Vec<WindowFrame>,
    mut encoder: FrameEncoder,
  ) {
    let RunningGpu {
      gpu,
      runtime,
      frame_span,
      belt,
      ..
    } = running;
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
    let shots: Vec<ShotCopy> = self.take_capture_shots(&gpu.context.device, &windows);
    let bursts: Vec<BurstCopy> = self.take_burst_shots(&gpu.context.device, &windows, &frames);

    runtime.timer.set_enabled(self.settings.is_gpu_timed);
    runtime.timer.set_tag(self.periodic.get_frame());

    let recording: Instant = Instant::now();
    let mut retried: Vec<(RenderViewportId, CaptureReply)> = Vec::new();
    let recorded: XrfResult<ExecutedGraph> = self.record_frame(
      (gpu, runtime),
      &mut windows,
      &frames,
      (&mut captures, &shots, &bursts),
      &mut retried,
      started,
    );

    self.retry_captures(retried);

    let executed: ExecutedGraph = match recorded {
      Ok(executed) => executed,
      Err(error) => {
        self.abandon_frame((gpu, belt), encoder, (frames, &firsts), captures, &error);

        return;
      }
    };
    let waited: Duration = windows.iter().map(|window| window.acquire).sum();
    let groups: Vec<ExecutedGroup> = self.submit_frame(
      (gpu, runtime, frame_span, belt),
      executed,
      encoder,
      (recording, waited),
      &mut phases,
    );

    self.end_view_frames(frames, captures, (shots, bursts));

    let cpu: Duration = started.elapsed().saturating_sub(waited);

    self.present_windows(&gpu.context.queue, windows, phases, (cpu, span), &groups);

    profiling::finish_frame!();
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
        self.periodic.set_owner(PeriodicOwner::View(*id));
        scene.lights.offer_shadows(&mut self.periodic);
      }
    }

    self.periodic.grant();

    for (id, ..) in frames {
      let Some(viewport) = self.viewports.get_mut(id) else {
        continue;
      };
      let Some(scene) = viewport
        .shown
        .as_ref()
        .and_then(|(scene_id, _)| self.scenes.get(scene_id))
      else {
        continue;
      };
      let Some(view) = viewport.get_drawn_view_mut() else {
        continue;
      };

      view.commit_periodic(scene, &gpu.context.device, encoder, &self.periodic);
    }

    for (scene_id, id) in firsts {
      let (Some(scene), Some((_, view, _))) = (
        self.scenes.get_mut(scene_id),
        self.viewports.get(id).and_then(RenderViewport::get_drawn_view),
      ) else {
        continue;
      };

      view.commit_scene_shadows(scene, &gpu.context.device, encoder, &self.periodic);
    }
  }

  /// Readies the present's and the overlays' pipelines for each window's image format.
  fn prepare_composition(&mut self, gpu: &mut GpuState, windows: &[WindowFrame]) {
    for format in windows.iter().map(|window| window.format).collect::<HashSet<_>>() {
      match gpu.present.prepare(
        PipelineSource {
          cache: &gpu.pipelines,
          device: &gpu.context.device,
          shaders: &self.shaders,
        },
        format,
      ) {
        Ok(()) => self.composition_failures.succeed("Level viewport cannot be presented"),
        Err(error) => self
          .composition_failures
          .fail("Level viewport cannot be presented", &error),
      }

      match gpu.overlay.prepare(
        PipelineSource {
          cache: &gpu.pipelines,
          device: &gpu.context.device,
          shaders: &self.shaders,
        },
        format,
      ) {
        Ok(()) => self.composition_failures.succeed("Level overlays cannot be drawn"),
        Err(error) => self.composition_failures.fail("Level overlays cannot be drawn", &error),
      }
    }
  }

  /// Records the frame's graph and executes it: each viewport's frame, its passes timed as its own; each window's
  /// image, acquired only once every view's work is recorded (the wait for it, while the GPU still draws the last
  /// frame, comes after the frame's own work on the render thread), a capture of a window that gave no image put back
  /// for the next frame; then each window composed and the captures copied out of it.
  fn record_frame(
    &mut self,
    (gpu, runtime): (&GpuState, &mut GraphRuntime),
    windows: &mut [WindowFrame],
    frames: &[ViewFrame],
    (captures, shots, bursts): (&mut Vec<CaptureCopy>, &[ShotCopy], &[BurstCopy]),
    retried: &mut Vec<(RenderViewportId, CaptureReply)>,
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
      let Some((scene_id, view, binding)) = viewport.get_drawn_view() else {
        continue;
      };
      let Some(scene) = self.scenes.get(&scene_id) else {
        continue;
      };

      graph.begin_owner(viewport.get_graph_owner());

      let output: Option<SceneOutput> = view.record(
        scene,
        GraphDeclaration::new(&mut graph, &mut bindings, &mut *runtime),
        gpu.get_level_passes(&self.shaders),
        binding,
        &gpu.textures,
        frame,
      );

      outputs.extend(output.map(|output| (*id, output)));
    }

    for window in windows.iter_mut() {
      // A window showing nothing draws its viewports' captures alone and takes no image to compose them into.
      if let Some(surface) = gpu.windows.get(&window.key).filter(|_| window.is_shown) {
        window.acquire(surface);
      }

      targets.push(
        window
          .image
          .as_ref()
          .map(|image| image.texture.create_view(&Default::default())),
      );
    }
    // A screenshot is drawn apart from its window, so it needs no image of it, only a scene drawn.
    let (kept, refused): (Vec<_>, Vec<_>) = std::mem::take(captures).into_iter().partition(|(id, capture, _)| {
      capture.is_screenshot()
        || windows
          .iter()
          .any(|window| window.image.is_some() && window.drawn.iter().any(|(drawn, ..)| drawn == id))
    });
    let (kept, undrawn): (Vec<_>, Vec<_>) = kept
      .into_iter()
      .partition(|(id, capture, _)| !capture.is_screenshot() || outputs.contains_key(id));

    *captures = kept;

    // A window that gave no image this frame, minimised or out of time, is captured on the next frame it presents.
    retried.extend(refused.into_iter().map(|(id, _, reply)| (id, reply)));

    for (id, _, reply) in undrawn {
      let _ = reply.send(Err(XrfError::new_invalid_error(format!(
        "Viewport {} draws no scene to take a screenshot of",
        id.0
      ))));
    }

    graph.begin_owner(GraphOwner::Frame);

    for (window, target) in windows.iter().zip(&targets) {
      let (Some(target), Some(grid)) = (target, gpu.get_grid(window.format)) else {
        continue;
      };
      // A viewport capturing is left to the backdrop: its frames are the capture's shots.
      let composed: Vec<ComposedView<'_>> = window
        .drawn
        .iter()
        .filter(|(id, ..)| !self.viewports.get(id).is_some_and(RenderViewport::is_capturing))
        .filter_map(|(id, rect, shown)| Self::compose_view(&self.viewports, gpu, (*id, *rect, *shown), outputs.get(id)))
        .collect();
      let backdrop: Option<&WindowBackdrop> = gpu
        .windows
        .get(&window.key)
        .and_then(RenderWindow::get_backdrop)
        .filter(|_| window.is_backdrop_painted);

      window.declare(
        GraphDeclaration::new(&mut graph, &mut bindings, &mut *runtime),
        (target, backdrop),
        composed,
        (&gpu.backdrop, &gpu.present, &gpu.overlay, grid),
        captures,
      );
    }

    for (id, capture, _) in captures.iter().filter(|(_, capture, _)| capture.is_screenshot()) {
      Self::declare_screenshot(
        GraphDeclaration::new(&mut graph, &mut bindings, &mut *runtime),
        gpu,
        &self.viewports,
        (*id, capture),
        outputs.get(id),
      );
    }

    for (id, capture) in shots {
      Self::declare_capture_shot(
        GraphDeclaration::new(&mut graph, &mut bindings, &mut *runtime),
        gpu,
        &self.viewports,
        (*id, capture),
        outputs.get(id),
      );
    }

    for (id, shot) in bursts {
      Self::declare_burst_shot(
        GraphDeclaration::new(&mut graph, &mut bindings, &mut *runtime),
        gpu,
        &self.viewports,
        (*id, shot),
        outputs.get(id),
      );
    }

    let graph_report: &mut Option<(Instant, GraphReport)> = &mut self.graph_report;
    #[cfg(debug_assertions)]
    let jitter_guard: &mut JitterGuard = &mut self.jitter_guard;

    graph.compile(&self.settings.graph.to_options()).and_then(|compiled| {
      if graph_report
        .as_ref()
        .is_none_or(|(described, _)| started.duration_since(*described) >= FrameStatistics::REPORT_INTERVAL)
      {
        let report: GraphReport = compiled.describe();

        #[cfg(debug_assertions)]
        jitter_guard.watch(&report);
        *graph_report = Some((started, report));
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
      present: output.map(|output| Self::to_present_parameters(gpu, output)),
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
      helpers: output.and(level).map(SceneView::get_helper_draws).unwrap_or_default(),
      rect,
      shown,
    })
  }

  /// What the present pass reads of a viewport's frame (`output`).
  pub(super) fn to_present_parameters<'a>(gpu: &'a GpuState, output: &SceneOutput) -> PresentParameters<'a> {
    PresentParameters {
      scene: output.scene,
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
      mark_target: output.marks,
    }
  }
}
