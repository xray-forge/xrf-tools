use std::collections::{HashMap, HashSet};
use std::sync::{Arc, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use xrf_dlss::Dlss;
use xrf_renderer_core::{GraphOwner, GraphPassTime, GraphTaggedTime};

use crate::context::gpu_requirements::can_trace;
use crate::contract::render_dlss_availability::RenderDlssAvailability;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::gpu_memory::GpuMemory;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_input::RenderViewInput;
use crate::host::render_world::RenderWorld;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_view::SceneView;
use crate::scene::texture::sky_texture_requests::SkyTextureRequests;
use crate::thread::running_gpu::RunningGpu;
use crate::viewport::viewport_device::ViewportDevice;

use super::{ALLOCATOR_REPORT_EVERY, RenderThread, TEXTURE_SWEEP};

impl RenderThread {
  /// One frame: the world advanced to the frames' clock, the shaders reloaded where they changed, the scenes fed what
  /// the world posted (whether or not any window can be drawn, so no post is lost), each viewport shown the scene it
  /// asks for, the scenes no one needs let go, the textures kept, every window drawn, then the reports.
  pub(super) fn frame(&mut self) {
    profiling::scope!("frame");

    let now: Instant = Instant::now();
    let delta: Duration = now.duration_since(self.last_frame);

    self.last_frame = now;
    self.pacer.begin(now, self.settings.frame_rate.get_interval());

    let (clock, clock_delta): (Instant, Duration) = self.tick_clock(now);
    let scene_frames: HashMap<RenderSceneId, RenderSceneFrame> = self.advance_world(clock, clock_delta);

    if self.shader_watcher.take_changed() {
      let changed: Vec<&'static str> = self.shaders.reload();

      if !changed.is_empty()
        && let Some(RunningGpu { gpu, .. }) = &mut self.running
      {
        gpu.refresh(&self.shaders, &changed);
      }
    }

    let Some((mut encoder, span)) = self.begin_frame_work(delta) else {
      return;
    };
    let live: HashSet<RenderSceneId> = self.ingest_scenes(scene_frames, &mut encoder, clock);

    self.show_asked_scenes();
    self.retain_scenes(&live);
    self.keep_textures(now);
    self.draw_windows((now.elapsed(), span), encoder);
    self.report_viewports(now);
    self.save_level_pipelines();
  }

  /// The time the frame begun at `wall` is drawn at, and how far that is from the last frame's: the time a burst draws
  /// its scene at while one draws, else the frames' own clock's. Every scene moves to it.
  fn tick_clock(&mut self, wall: Instant) -> (Instant, Duration) {
    let own: Instant = self.clock.tick(wall);
    let paced: Option<Instant> = self.viewports.values().find_map(|viewport| {
      let time: Duration = viewport.burst.as_ref().filter(|burst| !burst.is_ended())?.get_time()?;
      let scene: &LevelScene = self.scenes.get(&viewport.shown.as_ref()?.0)?;

      Some(scene.get_instant(time))
    });
    let now: Instant = paced.unwrap_or(own);
    let delta: Duration = now.saturating_duration_since(self.drawn_at);

    self.drawn_at = now;

    for scene in self.scenes.values_mut() {
      scene.set_now(now);
    }

    (now, delta)
  }

  /// Keeps the driver's pipelines once a level first stands drawn on this GPU, by when its passes made most of theirs.
  fn save_level_pipelines(&mut self) {
    let Some(running) = &mut self.running else {
      return;
    };

    if running.is_level_cache_saved || !self.scenes.values().any(|scene| scene.is_ready(&running.gpu.textures)) {
      return;
    }

    running.gpu.save_driver_cache(self.store.as_deref());
    running.is_level_cache_saved = true;
  }

  /// Runs the world inline before the frame: it moves each viewport's camera and weather, then streams each scene and
  /// plays what lives in it, and answers what each scene takes in. Without a GPU no scene could take in what the world
  /// posts, so the scenes wait for one rather than stream into nothing.
  fn advance_world(&mut self, now: Instant, delta: Duration) -> HashMap<RenderSceneId, RenderSceneFrame> {
    profiling::scope!("advance world");

    let mut world: MutexGuard<'_, dyn RenderWorld> = self.world.lock().unwrap_or_else(PoisonError::into_inner);

    for viewport in self.viewports.values_mut() {
      let mut skies: SkyTextureRequests<'_> = SkyTextureRequests {
        cache: self.running.as_mut().map(|running| &mut running.gpu.weather_textures),
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

    if self.running.is_none() {
      return HashMap::new();
    }

    let feedback: HashMap<RenderSceneId, RenderSceneFeedback> = self
      .scenes
      .iter_mut()
      .map(|(id, scene)| (*id, scene.take_feedback()))
      .collect();

    world.advance_scenes(now, feedback)
  }

  /// Opens the frame's encoder and grants its periodic work and streaming against one budget, the streaming's rounds
  /// first, from the timers read back since; answers the encoder and the GPU's milliseconds on the latest frame read
  /// back. None without a GPU.
  fn begin_frame_work(&mut self, delta: Duration) -> Option<(FrameEncoder, Option<f32>)> {
    let running: &mut RunningGpu = self.running.as_mut()?;
    let mut encoder: FrameEncoder = running.begin_encoder();
    let times: Vec<GraphTaggedTime> = running.runtime.timer.take_tagged();
    let spans: Vec<(u64, f32)> = running.frame_span.take();
    let span: Option<f32> = spans.last().map(|(_, span)| *span);
    let budget: f32 =
      self
        .periodic_budget
        .update((delta, span), self.settings.frame_rate, self.settings.periodic_budget);

    self.periodic.begin(budget, (&times, &spans));
    running.frame_span.begin(&mut encoder, self.periodic.get_frame());

    Some((encoder, span))
  }

  /// Feeds each scene what the world posted for it, making it where it is new, opening at `now` on the frames' clock; a
  /// scene keeps its acceleration structures only while a viewport traces rays in it
  /// (`RenderViewOptions::is_scene_traced`). Answers the scenes the world still holds.
  fn ingest_scenes(
    &mut self,
    mut scene_frames: HashMap<RenderSceneId, RenderSceneFrame>,
    encoder: &mut FrameEncoder,
    now: Instant,
  ) -> HashSet<RenderSceneId> {
    let live: HashSet<RenderSceneId> = scene_frames.keys().copied().collect();
    let traced: HashSet<RenderSceneId> = self
      .viewports
      .values()
      .filter(|viewport| viewport.options.is_scene_traced())
      .filter_map(|viewport| viewport.world.scene)
      .collect();
    let Some(RunningGpu { gpu, .. }) = &mut self.running else {
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
          now,
        )
      });

      scene.advance(
        (&gpu.context.device, &gpu.context.queue, &mut *encoder),
        (&mut gpu.textures, &mut gpu.weather_textures),
        &mut self.periodic,
        (frame, traced.contains(&id)),
      );
    }

    live
  }

  /// Shows each viewport the scene it asks for: a level at once, unless it is made again after a restart; a scene of
  /// models alone, or a level made again, once it can be drawn whole, the one before drawn until then, so a model
  /// swapped never leaves the viewport empty.
  fn show_asked_scenes(&mut self) {
    let Some(RunningGpu { gpu, .. }) = &self.running else {
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
          SceneView::new(&gpu.context.device, &gpu.context.queue, &gpu.view_layout),
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
    let Some(RunningGpu { gpu, .. }) = &mut self.running else {
      return;
    };

    if now.duration_since(self.textures_swept) >= TEXTURE_SWEEP {
      profiling::scope!("texture sweep");

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

    profiling::scope!("texture uploads");

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
    let Some(RunningGpu { gpu, runtime, .. }) = &mut self.running else {
      return;
    };
    let _ = gpu.context.device.poll(wgpu::PollType::Poll);
    let (dlss, ray_reconstruction): (RenderDlssAvailability, RenderDlssAvailability) =
      match (Dlss::IS_BUILT, &gpu.context.dlss) {
        (false, _) => (RenderDlssAvailability::Unbuilt, RenderDlssAvailability::Unbuilt),
        (true, Some(dlss)) => (
          RenderDlssAvailability::Available,
          if dlss.has_ray_reconstruction() {
            RenderDlssAvailability::Available
          } else {
            RenderDlssAvailability::Unsupported
          },
        ),
        (true, None) => (RenderDlssAvailability::Unsupported, RenderDlssAvailability::Unsupported),
      };
    let device: ViewportDevice<'_> = ViewportDevice {
      backend: gpu.context.backend,
      adapter: gpu.context.adapter_name.as_str(),
      has_ray_queries: can_trace(&gpu.context.device),
      dlss,
      ray_reconstruction,
    };

    let allocator: Option<&GpuMemory> = Self::read_allocator_report(
      &mut self.allocator_report,
      (&gpu.context.adapter, &gpu.context.device),
      now,
    );

    profiling::scope!("frame report");

    // The windows' own passes, composition and captures, are every viewport's: taken once a round any report is due in,
    // and handed to each viewport reporting, rather than to whichever reports first.
    let is_any_due: bool = self.viewports.values().any(|viewport| viewport.is_report_due(now));
    let composition: Vec<GraphPassTime> = if is_any_due {
      runtime.timer.take(GraphOwner::Frame)
    } else {
      Vec::new()
    };

    for viewport in self.viewports.values_mut() {
      viewport.answer_readbacks(&self.scenes);
      viewport.report(
        now,
        device,
        (allocator, &mut self.scenes),
        (
          &mut runtime.timer,
          &composition,
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

  /// What the GPU allocator holds: its last answer while younger than `ALLOCATOR_REPORT_EVERY`, else asked again and
  /// kept, its peak carried on.
  pub(super) fn read_allocator_report<'a>(
    report: &'a mut Option<(Instant, Option<GpuMemory>)>,
    gpu: (&wgpu::Adapter, &wgpu::Device),
    now: Instant,
  ) -> Option<&'a GpuMemory> {
    if report
      .as_ref()
      .is_none_or(|(asked, _)| now.duration_since(*asked) >= ALLOCATOR_REPORT_EVERY)
    {
      profiling::scope!("allocator report");

      let last: Option<&GpuMemory> = report.as_ref().and_then(|(_, held)| held.as_ref());
      let held: Option<GpuMemory> = GpuMemory::read(gpu, last);

      *report = Some((now, held));
    }

    report.as_ref().and_then(|(_, held)| held.as_ref())
  }
}
