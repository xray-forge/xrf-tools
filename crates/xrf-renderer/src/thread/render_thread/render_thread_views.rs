use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::capture::burst::burst_job::BurstJob;
use crate::capture::capture_job::CaptureJob;
use crate::capture::capture_shot::CaptureShot;
use crate::contract::view::output::render_debug_view::RenderDebugView;
use crate::contract::view::output::render_scale::RenderScale;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_owner::PeriodicOwner;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::timing::frame_phases::FramePhases;
use crate::host::weather::render_level_weather::RenderLevelWeather;
use crate::host::world::render_scene_id::RenderSceneId;
use crate::host::world::view::render_view_frame::RenderViewFrame;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::pass::view::view_binding::ViewBinding;
use crate::pass::view::view_motion::ViewMotion;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::page::helpers::view::helper_sources::HelperSources;
use crate::scene::level::view::record::readied_frame::ReadiedFrame;
use crate::scene::level::view::scene_asks::SceneAsks;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::level::view::view_lens::ViewLens;
use crate::scene::level::view::view_pick::ViewPick;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::static_scene::draw::static_selection::StaticSelection;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu_state::GpuState;
use crate::thread::views::prepared_view::PreparedView;
use crate::thread::views::readied_view::ReadiedView;
use crate::thread::views::readied_views::ReadiedViews;
use crate::thread::views::scene_recorder::SceneRecorder;
use crate::thread::views::scene_view_framing::SceneViewFraming;
use crate::thread::views::view_work::ViewWork;
use crate::viewport::pick::drawn_pick::DrawnPick;
use crate::viewport::pick::pending_pick::PendingPick;
use crate::viewport::pick::pick_unprojection::PickUnprojection;
use crate::viewport::render_viewport::RenderViewport;
use crate::viewport::viewport_parts::ViewportParts;
use crate::window::window_frame::WindowFrame;

use super::RenderThread;

/// A viewport a window draws this frame, by its whole rectangle, and the scene it shows, none where it shows none.
struct DrawnView {
  id: RenderViewportId,
  rect: RenderRect,
  scene: Option<RenderSceneId>,
}

impl RenderThread {
  /// Readies each drawn viewport's frame, with the pick it draws and what unprojects that pick's depth, in three phases:
  /// each view prepares from its scene and answers what it asks of it, its scene's driving viewport first, which
  /// steps the scene, or the first drawn where that one is not; each scene settles what its views asked; then each
  /// view finishes its frame. Answers the frames and each scene's first view to ready one, which records the scene's
  /// own work.
  pub(super) fn ready_views(
    &mut self,
    gpu: &mut GpuState,
    windows: &[WindowFrame],
    encoder: &mut FrameEncoder,
    phases: &mut FramePhases,
  ) -> ReadiedViews {
    // Every view samples the one texture cache, so it is filtered and loaded by the renderer's settings, never a
    // view's.
    gpu
      .textures
      .set_anisotropy(&gpu.context.device, self.settings.texture_filtering.get_anisotropy());
    gpu.textures.set_lod(self.settings.texture_lod);

    let mut drawn: Vec<DrawnView> = windows
      .iter()
      .flat_map(|window| window.drawn.iter())
      .map(|drawn| DrawnView {
        id: drawn.id,
        rect: drawn.rect,
        scene: self.viewports.get(&drawn.id).and_then(RenderViewport::get_shown_scene),
      })
      .collect();

    drawn.sort_by_key(|view| {
      !view
        .scene
        .and_then(|scene| self.scenes.get(&scene))
        .is_some_and(|scene| scene.get_driver() == Some(view.id))
    });

    // Each scene steps once a frame by its first view, which stands for the actor, before any view reads it; its
    // effects step on the workers while the views are readied, and its settle waits for them.
    let mut simulated: Vec<RenderSceneId> = Vec::new();

    for DrawnView {
      id,
      rect,
      scene: scene_id,
    } in &drawn
    {
      let (Some(scene_id), Some(viewport)) = (*scene_id, self.viewports.get(id)) else {
        continue;
      };

      if !simulated.contains(&scene_id)
        && let Some(scene) = self.scenes.get_mut(&scene_id)
      {
        let (lighting, _): (Cow<'_, RenderLighting>, Option<&Arc<RenderLevelWeather>>) =
          Self::get_view_lighting(viewport.get_options(), viewport.get_world());

        scene.begin_frame(
          &Self::get_camera_view(viewport, rect),
          viewport.get_options(),
          &lighting,
        );
        simulated.push(scene_id);
      }
    }

    let mut prepared: Vec<PreparedView> = Vec::new();

    for DrawnView {
      id,
      rect,
      scene: scene_id,
    } in &drawn
    {
      let Some(viewport) = self.viewports.get_mut(id) else {
        continue;
      };
      let scene: Option<&LevelScene> = scene_id.and_then(|scene| self.scenes.get(&scene));

      self.periodic.set_owner(PeriodicOwner::View(*id));

      let view: Option<PreparedView> = Self::ready_view(
        &mut ViewWork {
          gpu: &mut *gpu,
          shaders: &self.shaders,
          encoder: &mut *encoder,
          periodic: &mut self.periodic,
          phases: &mut *phases,
        },
        viewport,
        scene,
        rect,
      );

      // A view that draws nothing this frame draws none of what it offered.
      if view.as_ref().is_none_or(|view| view.frame.is_none()) {
        self.periodic.withdraw(PeriodicOwner::View(*id));
      }

      prepared.extend(view);
    }

    let finishing: Instant = Instant::now();
    let mut readied: ReadiedViews = ReadiedViews {
      frames: Vec::new(),
      recorders: Vec::new(),
    };

    self.settle_scenes(&prepared);

    for PreparedView {
      id,
      scene: scene_id,
      asks,
      options,
      output,
      frame,
    } in prepared
    {
      let view: Option<&mut SceneView> = self.viewports.get_mut(&id).and_then(RenderViewport::get_drawn_view_mut);

      if let (Some(view), Some(scene)) = (view, self.scenes.get(&scene_id)) {
        view.finish_prepare(scene, &gpu.context.device, &mut *encoder, &options, &asks);
        view.finish_present(scene, &gpu.textures, &options, output);
      }

      let Some(mut frame) = frame else {
        continue;
      };

      if readied.recorders.iter().all(|recorder| recorder.scene != scene_id) {
        frame.frame.is_scene_recorder = true;
        readied.recorders.push(SceneRecorder {
          scene: scene_id,
          viewport: id,
        });
      }

      readied.frames.push(frame);
    }

    phases.prepare += finishing.elapsed();

    readied
  }

  /// Settles what each scene's views asked of it this frame, in the order they prepared: its driving viewport's first.
  fn settle_scenes(&mut self, prepared: &[PreparedView]) {
    let mut settled: Vec<RenderSceneId> = Vec::new();

    for view in prepared {
      if settled.contains(&view.scene) {
        continue;
      }

      settled.push(view.scene);

      if let Some(scene) = self.scenes.get_mut(&view.scene) {
        scene.settle(
          prepared
            .iter()
            .filter(|asking| asking.scene == view.scene)
            .map(|asking| &asking.asks),
        );
      }
    }
  }

  /// The sizes a viewport's frame showing a scene is resolved at and drawn at over its rectangle `rect`: the settings'
  /// output resolution first, then the render scale's share of it. Each side stays within `most`, the largest texture
  /// the device makes.
  fn get_scene_sizes(options: &RenderViewOptions, rect: &RenderRect, most: u32) -> ((u32, u32), RenderRect) {
    Self::fit_view_sizes(
      options.output.get_output_size((rect.width, rect.height)),
      options.output.upscaling.scale,
      rect,
      most,
    )
  }

  /// The same for a viewport's frame showing no scene, which is drawn but at the viewport's size.
  fn get_grid_sizes(rect: &RenderRect, most: u32) -> ((u32, u32), RenderRect) {
    Self::fit_view_sizes((rect.width, rect.height), RenderScale::Native, rect, most)
  }

  /// The sizes a frame `resolved` across and down, each side within `most`, is resolved at and drawn at by
  /// `render_scale` over its rectangle `rect`.
  fn fit_view_sizes(
    resolved: (u32, u32),
    render_scale: RenderScale,
    rect: &RenderRect,
    most: u32,
  ) -> ((u32, u32), RenderRect) {
    let resolved: (u32, u32) = (resolved.0.min(most), resolved.1.min(most));

    (
      resolved,
      RenderRect {
        width: render_scale.get_drawn(resolved.0),
        height: render_scale.get_drawn(resolved.1),
        ..*rect
      },
    )
  }

  /// Asks for the heights where a view's enhanced parallax or the heights' debug view reads them.
  fn ask_heights(textures: &mut TextureCache, options: &RenderViewOptions) {
    if options.features.parallax.is_enhanced() || options.mode.debug_view == RenderDebugView::Heights {
      textures.want_heights();
    }
  }

  /// Readies a view's graph frame, with one pick a frame drawn from this frame's culled clusters while a readback is
  /// free for it: the camera narrowed to the pick's texel of the viewport's rectangle as the view is framed, at the
  /// viewport's `scale`. The `pick` taken for this frame is left there where no readback is free for it, to wait for the
  /// next frame.
  fn ready_pick(
    work: &mut ViewWork<'_>,
    id: RenderViewportId,
    view_state: &mut SceneView,
    pick: &mut Option<PendingPick>,
    framing: &SceneViewFraming<'_>,
    scale: f32,
  ) -> Option<ReadiedView> {
    let (view, rect): (&CameraView, &RenderRect) = (&framing.view, &framing.rect);
    let switches: Vec4 = CameraUniform::get_switches(framing.options);
    let (device, view_layout): (&wgpu::Device, &wgpu::BindGroupLayout) =
      (&work.gpu.context.device, &work.gpu.view_layout);
    let encoder: &mut FrameEncoder = &mut *work.encoder;
    let drawn: Option<DrawnPick> = pick.take().map(|pick| {
      let ndc: Vec2 = Vec2::new(
        (pick.x * scale + 0.5) / rect.width as f32 * 2.0 - 1.0,
        1.0 - (pick.y * scale + 0.5) / rect.height as f32 * 2.0,
      );

      DrawnPick {
        pick,
        unprojection: PickUnprojection { view: *view, ndc },
      }
    });
    let camera: Option<CameraUniform> = drawn.as_ref().map(|drawn| {
      let narrowed: CameraView = view.narrow_to(
        drawn.unprojection.ndc,
        Vec2::new(2.0 / rect.width as f32, 2.0 / rect.height as f32),
      );
      let pixel: RenderRect = RenderRect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
      };

      CameraUniform::new(&narrowed, pixel, switches)
    });
    let readied: Option<ReadiedFrame> = view_state.begin_frame(
      device,
      encoder,
      drawn.as_ref().zip(camera.as_ref()).map(|(drawn, camera)| ViewPick {
        camera,
        view_layout,
        is_through: drawn.pick.is_through,
      }),
    );

    match (readied, drawn) {
      (Some(frame), Some(drawn)) if frame.pick_slot.is_some() => Some(ReadiedView {
        viewport: id,
        frame,
        pick: Some(drawn),
      }),
      (readied, drawn) => {
        *pick = drawn.map(|drawn| drawn.pick);

        readied.map(|frame| ReadiedView {
          viewport: id,
          frame,
          pick: None,
        })
      }
    }
  }

  /// What a viewport's camera sees over `rect`, its whole rectangle: a lit and fogged level ends where its fog is
  /// total, or at the weather's far plane, as the engine's does. A capture laid out nowhere has no rectangle, and its
  /// camera sees as wide as high.
  fn get_camera_view(viewport: &RenderViewport, rect: &RenderRect) -> CameraView {
    viewport.get_world().camera.to_view(
      rect.width.max(1) as f32 / rect.height.max(1) as f32,
      Self::get_far_limit(viewport),
    )
  }

  /// How far a viewport's camera sees at most: where a lit and fogged level's fog is total, else without end.
  pub(super) fn get_far_limit(viewport: &RenderViewport) -> f32 {
    let options: &RenderViewOptions = viewport.get_options();

    viewport
      .get_world()
      .lighting
      .fog
      .filter(|_| options.mode.is_lit && options.show.is_fogged)
      .map_or(f32::INFINITY, |fog| fog.get_total_distance())
  }

  /// Readies one viewport's frame of the scene it shows: writes its camera, asks for the textures its weather draws
  /// with, prepares its frame, and readies its graph's frame with a pick when one waits; none where it shows no scene or
  /// draws nothing this frame. `rect` is its whole rectangle.
  fn ready_view(
    work: &mut ViewWork<'_>,
    viewport: &mut RenderViewport,
    scene: Option<&LevelScene>,
    rect: &RenderRect,
  ) -> Option<PreparedView> {
    if viewport.is_capturing() {
      return Self::ready_capture_view(work, viewport, scene);
    }

    let burst: Option<&BurstJob> = viewport.get_burst();

    // A burst waiting for its frames back draws nothing, its time held.
    if burst.is_some_and(|burst| !burst.is_drawing()) {
      return None;
    }

    let reference: Option<(u32, u32)> = burst.and_then(BurstJob::get_reference);
    let is_restarted: bool = burst.is_some_and(BurstJob::is_starting);
    let id: RenderViewportId = viewport.get_id();
    let device: &wgpu::Device = &work.gpu.context.device;
    let scale: f32 = viewport.get_scale();
    let view: CameraView = Self::get_camera_view(viewport, rect);
    let mut options: RenderViewOptions = ViewLens::fit(viewport.get_options(), &view).into_owned();

    // A reference is drawn at the size its frame is resolved at, as a resolve converges to.
    if reference.is_some() {
      options.output.upscaling.scale = RenderScale::Native;
    }

    let switches: Vec4 = CameraUniform::get_switches(&options);
    let most: u32 = device.limits().max_texture_dimension_2d;
    let (resolved, drawn_rect): ((u32, u32), RenderRect) = match scene {
      Some(_) => Self::get_scene_sizes(&options, rect, most),
      None => Self::get_grid_sizes(rect, most),
    };
    // One pick a frame, taken while it draws a scene for the pick to land in, and put back where it is not drawn.
    let mut pick: Option<PendingPick> = scene
      .filter(|_| viewport.get_shown_view().is_some())
      .and_then(|_| viewport.take_pick());
    let ViewportParts {
      world,
      overlays,
      selection,
      scene: scene_id,
      view: view_state,
      binding,
      ..
    } = viewport.split();
    let binding: &ViewBinding = binding.get_or_insert_with(|| ViewBinding::new(device, &work.gpu.view_layout));
    let (Some(scene), Some(scene_id), Some(view_state)) = (scene, scene_id, view_state) else {
      // Nothing to draw but its grid: its camera alone.
      let unjittered: Mat4 = view.get_view_projection();

      binding.write(
        work.encoder,
        &CameraUniform::new(&view, drawn_rect, switches)
          .with_wireframe(options.mode.is_wireframe)
          .with_surface_color(options.mode.surface_color)
          .with_motion(ViewMotion::still(unjittered, unjittered))
          .with_asset_view(&options, drawn_rect.height as f32 / rect.height.max(1) as f32),
      );

      return None;
    };
    let (lighting, weather): (Cow<'_, RenderLighting>, Option<&Arc<RenderLevelWeather>>) =
      Self::get_view_lighting(&options, world);

    if is_restarted {
      view_state.restart_motion();
    }

    view_state.sample_reference(reference);

    let framing: SceneViewFraming<'_> = SceneViewFraming {
      view,
      rect: *rect,
      resolved,
      drawn: drawn_rect,
      options: &options,
      lighting: &lighting,
      weather,
      overlays: Some(overlays),
      selection,
      weather_rate: world.clock_rate,
    };

    let asks: SceneAsks = Self::prepare_scene_view(work, scene, &mut *view_state, binding, &framing);
    let frame: Option<ReadiedView> = Self::ready_pick(work, id, view_state, &mut pick, &framing, scale);
    let prepared: PreparedView = PreparedView {
      id,
      scene: scene_id,
      asks,
      options,
      output: *rect,
      frame,
    };

    if let Some(pick) = pick {
      viewport.return_pick(pick);
    }

    Some(prepared)
  }

  /// Readies the frame of a viewport capturing: its capture's view drawn through the shot it stands at, with the
  /// capture's look and lighting, over the viewport's world frame; none while its last shots come back.
  fn ready_capture_view(
    work: &mut ViewWork<'_>,
    viewport: &mut RenderViewport,
    scene: Option<&LevelScene>,
  ) -> Option<PreparedView> {
    let id: RenderViewportId = viewport.get_id();
    let scene: &LevelScene = scene?;
    let ViewportParts {
      capture,
      world,
      options,
      scene: scene_id,
      ..
    } = viewport.split();
    let scene_id: RenderSceneId = scene_id?;
    let capture: &mut CaptureJob = capture?;
    let shot: CaptureShot = capture.get_drawn_shot()?;
    let view: CameraView = shot.get_view();
    let look: RenderViewOptions = if capture.is_metering() {
      shot.apply_metering_look(options)
    } else {
      shot.apply_look(options)
    };
    let options: RenderViewOptions = ViewLens::fit(&look, &view).into_owned();
    let device: &wgpu::Device = &work.gpu.context.device;
    let queue: &wgpu::Queue = &work.gpu.context.queue;
    let (width, height): (u32, u32) = shot.get_drawn_size();
    let rect: RenderRect = RenderRect {
      x: 0,
      y: 0,
      width,
      height,
    };
    let (resolved, drawn_rect): ((u32, u32), RenderRect) =
      Self::get_scene_sizes(&options, &rect, device.limits().max_texture_dimension_2d);
    let view_layout: &wgpu::BindGroupLayout = &work.gpu.view_layout;
    let (view_state, binding): (&mut SceneView, &ViewBinding) = capture.get_view_mut(|| {
      (
        SceneView::new(device, queue, view_layout),
        ViewBinding::new(device, view_layout),
      )
    });
    let (lighting, weather): (Cow<'_, RenderLighting>, Option<&Arc<RenderLevelWeather>>) =
      Self::get_view_lighting(&options, world);
    let lighting: Cow<'_, RenderLighting> = shot.apply_lighting(lighting);

    let asks: SceneAsks = Self::prepare_scene_view(
      work,
      scene,
      &mut *view_state,
      binding,
      &SceneViewFraming {
        view,
        rect,
        resolved,
        drawn: drawn_rect,
        options: &options,
        lighting: &lighting,
        weather,
        overlays: None,
        selection: None,
        weather_rate: world.clock_rate,
      },
    );
    let frame: Option<ReadiedView> = view_state
      .begin_frame(&work.gpu.context.device, work.encoder, None)
      .map(|frame| ReadiedView {
        viewport: id,
        frame,
        pick: None,
      });

    Some(PreparedView {
      id,
      scene: scene_id,
      asks,
      options,
      output: rect,
      frame,
    })
  }

  /// Prepares one view of a scene for its frame: writes its camera, its jitter and motion with it, asks for the
  /// textures its weather draws with and its overlays where it has them, prepares its features and offers its periodic
  /// work; what loading and preparing take is added to the frame's phases. Answers what it asks of the scene.
  fn prepare_scene_view(
    work: &mut ViewWork<'_>,
    scene: &LevelScene,
    view_state: &mut SceneView,
    binding: &ViewBinding,
    framing: &SceneViewFraming<'_>,
  ) -> SceneAsks {
    let gpu: &mut GpuState = &mut *work.gpu;
    let shaders: &ShaderLibrary = work.shaders;
    let encoder: &mut FrameEncoder = &mut *work.encoder;
    let periodic: &mut PeriodicScheduler = &mut *work.periodic;
    let phases: &mut FramePhases = &mut *work.phases;
    let SceneViewFraming {
      view,
      rect,
      resolved,
      drawn: drawn_rect,
      options,
      lighting,
      weather,
      overlays,
      selection,
      weather_rate,
    } = *framing;
    let device: &wgpu::Device = &gpu.context.device;
    let queue: &wgpu::Queue = &gpu.context.queue;
    let switches: Vec4 = CameraUniform::get_switches(options);
    // A temporal resolve's jitter moves every scene pass's samples, never the view its history is measured by.
    let jitter: Vec2 = view_state.next_jitter(
      options,
      resolved.0 as f32 / drawn_rect.width.max(1) as f32,
      gpu.context.dlss.get(),
    );
    let drawn: CameraView = view.jittered(jitter, Vec2::new(drawn_rect.width as f32, drawn_rect.height as f32));
    let motion: ViewMotion = view_state.next_motion(&view, drawn.get_view_projection());
    let selected: Option<StaticSelection> = view_state.resolve_selection(scene, selection).cloned();
    // The enhanced parallax shadows its relief towards the sun, as the view sees it.
    let to_sun: Vec3 = drawn.view.transform_vector3(-lighting.get_sun_direction());

    binding.write(
      encoder,
      &CameraUniform::new(&drawn, drawn_rect, switches)
        .with_parallax(&options.features.parallax, to_sun)
        .with_terrain(&options.features.terrain)
        .with_selection(selected.as_ref())
        .with_wireframe(options.mode.is_wireframe)
        .with_surface_color(options.mode.surface_color)
        .with_linear_scene(options.is_scene_linear())
        .with_physical_light(options)
        .with_light_view(options.mode.debug_view)
        .with_texture_bias(view_state.get_texture_bias())
        .with_motion(motion)
        .with_asset_view(options, drawn_rect.height as f32 / rect.height.max(1) as f32),
    );

    let loading: Instant = Instant::now();

    {
      profiling::scope!("load");

      if let Some(overlays) = overlays {
        view_state.set_overlays(scene, device, &overlays.overlays, overlays.version, selection);
      }

      view_state.set_helpers(
        device,
        HelperSources {
          helpers: &scene.helpers,
          sounding: &scene.sounding,
          statics: &scene.statics,
          options: &options.helpers,
          marks_layout: gpu.scene.gbuffer.get_marks_layout(),
        },
      );

      view_state.request_textures(scene, &mut gpu.weather_textures, lighting, weather, options);
      Self::ask_heights(&mut gpu.textures, options);
    }
    phases.load += loading.elapsed();

    let preparing: Instant = Instant::now();
    let asks: SceneAsks = {
      profiling::scope!("prepare");

      let asks: SceneAsks = view_state.prepare(
        scene,
        encoder,
        &ViewPrepare {
          device,
          queue,
          dlss: gpu.context.dlss.get(),
          passes: gpu.get_level_passes(shaders),
          view: &view,
          drawn: (drawn_rect.width, drawn_rect.height),
          resolved,
          output: rect,
          options,
          lighting,
          weather,
          weather_textures: gpu.weather_textures.scoped(scene.source.get_texture_scope()),
          weather_rate,
        },
      );
      view_state.offer_periodic(scene, device, periodic, &gpu.view_layout);

      asks
    };

    phases.prepare += preparing.elapsed();

    asks
  }

  /// What a view is lit by and the weather it plays: an asset viewer's rig and none, else its world's frame's.
  fn get_view_lighting<'a>(
    options: &RenderViewOptions,
    world: &'a RenderViewFrame,
  ) -> (Cow<'a, RenderLighting>, Option<&'a Arc<RenderLevelWeather>>) {
    match options.asset.lighting.as_ref() {
      Some(rig) => (Cow::Owned(RenderLighting::for_asset(rig)), None),
      None => (Cow::Borrowed(&world.lighting), world.weather.as_ref()),
    }
  }
}
