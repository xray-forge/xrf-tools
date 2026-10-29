use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::capture::burst::burst_job::BurstJob;
use crate::capture::capture_job::CaptureJob;
use crate::capture::capture_shot::CaptureShot;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::frame_phases::FramePhases;
use crate::frame::periodic::periodic_owner::PeriodicOwner;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_frame::RenderViewFrame;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_view::SceneView;
use crate::scene::level::view_lens::ViewLens;
use crate::scene::level::view_prepare::ViewPrepare;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu_state::GpuState;
use crate::thread::scene_view_framing::SceneViewFraming;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::render_viewport::RenderViewport;
use crate::window::window_frame::WindowFrame;

use super::{RenderThread, ViewFrame};

impl RenderThread {
  /// Readies each drawn viewport's frame, with the pick it draws and what unprojects that pick's depth: its scene's
  /// driving viewport first, which simulates the scene and declares its work, or the first drawn where that one is not.
  /// Answers the frames and each scene's first view, which draws the scene's own work.
  pub(super) fn ready_views(
    &mut self,
    gpu: &mut GpuState,
    windows: &[WindowFrame],
    encoder: &mut FrameEncoder,
    phases: &mut FramePhases,
  ) -> (Vec<ViewFrame>, Vec<(RenderSceneId, RenderViewportId)>) {
    let mut frames: Vec<ViewFrame> = Vec::new();

    // Every view samples the one texture cache, so it is filtered and loaded by the renderer's settings, never a
    // view's.
    gpu
      .textures
      .set_anisotropy(&gpu.context.device, self.settings.texture_filtering.get_anisotropy());
    gpu.textures.set_lod(self.settings.texture_lod);

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

    // Each scene's effects step once a frame by the camera of its first view, which stands for the actor, on the
    // workers while the views are readied; the first fill of them waits for the step. Its wetness moves on once too,
    // before any view reads it.
    let mut simulated: Vec<RenderSceneId> = Vec::new();

    for (id, rect, scene_id) in &drawn {
      let (Some(scene_id), Some(viewport)) = (*scene_id, self.viewports.get(id)) else {
        continue;
      };

      if !simulated.contains(&scene_id)
        && let Some(scene) = self.scenes.get_mut(&scene_id)
      {
        let time: Duration = scene.get_time();

        scene
          .particles
          .start_simulation(&Self::get_camera_view(viewport, rect), &viewport.options, time);
        Self::advance_wetness(scene, &self.viewports);
        simulated.push(scene_id);
      }
    }

    let mut firsts: Vec<(RenderSceneId, RenderViewportId)> = Vec::new();

    for (id, rect, scene_id) in &drawn {
      let scene_id: Option<RenderSceneId> = *scene_id;
      let Some(viewport) = self.viewports.get_mut(id) else {
        continue;
      };
      let is_first: bool = scene_id.is_some_and(|scene| firsts.iter().all(|(seen, _)| *seen != scene));
      let scene: Option<&mut LevelScene> = scene_id.and_then(|scene| self.scenes.get_mut(&scene));

      self.periodic.set_owner(PeriodicOwner::View(*id));

      match Self::ready_view(
        (gpu, &self.shaders),
        (viewport, scene, is_first),
        rect,
        (&mut *encoder, &mut self.periodic),
        (&mut phases.load, &mut phases.prepare),
      ) {
        Some(mut frame) => {
          if let (true, Some(scene)) = (is_first, scene_id) {
            frame.1.is_scene_first = true;
            firsts.push((scene, *id));
          }

          frames.push(frame);
        }
        // A view that draws nothing this frame draws none of what it offered.
        None => self.periodic.withdraw(PeriodicOwner::View(*id)),
      }
    }

    (frames, firsts)
  }

  /// The sizes a viewport's frame is resolved at and drawn at over its rectangle `rect`: the settings' output
  /// resolution first, then the render scale's share of it, where it shows a scene (`is_scened`); nothing else is drawn
  /// but at the viewport's size. Each side stays within `most`, the largest texture the device makes.
  fn get_view_sizes(
    options: &RenderViewOptions,
    rect: &RenderRect,
    is_scened: bool,
    most: u32,
  ) -> ((u32, u32), RenderRect) {
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

  /// What a viewport's camera sees over `rect`, its whole rectangle: a lit and fogged level ends where its fog is
  /// total, or at the weather's far plane, as the engine's does.
  fn get_camera_view(viewport: &RenderViewport, rect: &RenderRect) -> CameraView {
    viewport
      .world
      .camera
      .to_view(rect.width as f32 / rect.height as f32, Self::get_far_limit(viewport))
  }

  /// How far a viewport's camera sees at most: where a lit and fogged level's fog is total, else without end.
  pub(super) fn get_far_limit(viewport: &RenderViewport) -> f32 {
    viewport
      .world
      .lighting
      .fog
      .filter(|_| viewport.options.mode.is_lit && viewport.options.show.is_fogged)
      .map_or(f32::INFINITY, |fog| fog.get_total_distance())
  }

  /// Readies one viewport's frame of the scene it shows: writes its camera, asks for the textures its weather draws
  /// with, prepares its frame, and readies its graph's frame with a pick when one waits; none where it shows no scene.
  /// `rect` is its whole rectangle; what loading and preparing take is added to `load` and `prepare`.
  fn ready_view(
    (gpu, shaders): (&mut GpuState, &ShaderLibrary),
    (viewport, scene, is_scene_first): (&mut RenderViewport, Option<&mut LevelScene>, bool),
    rect: &RenderRect,
    (encoder, periodic): (&mut FrameEncoder, &mut PeriodicScheduler),
    (load, prepare): (&mut Duration, &mut Duration),
  ) -> Option<ViewFrame> {
    if viewport.is_capturing() {
      return Self::ready_capture_view(
        (gpu, shaders),
        (viewport, scene, is_scene_first),
        (encoder, periodic),
        (load, prepare),
      );
    }

    let burst: Option<&BurstJob> = viewport.burst.as_ref().filter(|burst| !burst.is_ended());

    // A burst waiting for its frames back draws nothing, its time held.
    if burst.is_some_and(|burst| !burst.is_drawing()) {
      return None;
    }

    let reference: Option<(u32, u32)> = burst.and_then(BurstJob::get_reference);
    let is_restarted: bool = burst.is_some_and(BurstJob::is_starting);
    let id: RenderViewportId = viewport.id;
    let device: &wgpu::Device = &gpu.context.device;
    let scale: f32 = viewport.get_scale();
    let view: CameraView = Self::get_camera_view(viewport, rect);
    let binding: &ViewBinding = viewport
      .binding
      .get_or_insert_with(|| ViewBinding::new(device, &gpu.view_layout));

    let mut options: RenderViewOptions = ViewLens::fit(&viewport.options, &view).into_owned();

    // A reference is drawn at the size its frame is resolved at, as a resolve converges to.
    if reference.is_some() {
      options.output.upscaling.scale = RenderScale::Native;
    }

    let switches: Vec4 = CameraUniform::get_switches(&options);
    let (resolved, drawn_rect): ((u32, u32), RenderRect) = Self::get_view_sizes(
      &options,
      rect,
      scene.is_some(),
      device.limits().max_texture_dimension_2d,
    );
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
    let (lighting, weather): (Cow<'_, RenderLighting>, Option<&Arc<RenderLevelWeather>>) =
      Self::get_view_lighting(&options, world);

    if is_restarted {
      view_state.restart_motion();
    }

    view_state.sample_reference(reference);

    Self::prepare_scene_view(
      (gpu, shaders),
      (scene, &mut *view_state, binding, is_scene_first),
      &SceneViewFraming {
        view,
        rect: *rect,
        resolved,
        drawn: drawn_rect,
        options: &options,
        lighting: &lighting,
        weather,
        overlays: Some((overlays, *overlays_version)),
        selection: selection.as_ref(),
        weather_rate: world.clock_rate,
      },
      (encoder, periodic),
      (load, prepare),
    );

    let device: &wgpu::Device = &gpu.context.device;

    Self::ready_pick(
      (id, view_state, picks),
      (device, encoder, &gpu.view_layout),
      (&view, rect, scale, switches),
    )
  }

  /// Readies the frame of a viewport capturing: its capture's view drawn through the shot it stands at, with the
  /// capture's look and lighting, over the viewport's world frame; none while its last shots come back.
  fn ready_capture_view(
    (gpu, shaders): (&mut GpuState, &ShaderLibrary),
    (viewport, scene, is_scene_first): (&mut RenderViewport, Option<&mut LevelScene>, bool),
    (encoder, periodic): (&mut FrameEncoder, &mut PeriodicScheduler),
    (load, prepare): (&mut Duration, &mut Duration),
  ) -> Option<ViewFrame> {
    let id: RenderViewportId = viewport.id;
    let scene: &mut LevelScene = scene?;
    let RenderViewport {
      capture,
      world,
      options,
      ..
    } = viewport;
    let capture: &mut CaptureJob = capture.as_mut()?;
    let shot: CaptureShot = capture.get_drawn_shot()?;
    let view: CameraView = shot.get_view();
    let options: RenderViewOptions =
      ViewLens::fit(&shot.apply_look(options, capture.is_metering()), &view).into_owned();
    let device: &wgpu::Device = &gpu.context.device;
    let queue: &wgpu::Queue = &gpu.context.queue;
    let (width, height): (u32, u32) = shot.get_drawn_size();
    let rect: RenderRect = RenderRect {
      x: 0,
      y: 0,
      width,
      height,
    };
    let (resolved, drawn_rect): ((u32, u32), RenderRect) =
      Self::get_view_sizes(&options, &rect, true, device.limits().max_texture_dimension_2d);
    let view_layout: &wgpu::BindGroupLayout = &gpu.view_layout;
    let (view_state, binding): (&mut SceneView, &ViewBinding) = capture.get_view_mut(|| {
      (
        SceneView::new(device, queue, view_layout),
        ViewBinding::new(device, view_layout),
      )
    });
    let (lighting, weather): (Cow<'_, RenderLighting>, Option<&Arc<RenderLevelWeather>>) =
      Self::get_view_lighting(&options, world);
    let lighting: Cow<'_, RenderLighting> = shot.apply_lighting(lighting);

    Self::prepare_scene_view(
      (gpu, shaders),
      (scene, &mut *view_state, binding, is_scene_first),
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
      (encoder, periodic),
      (load, prepare),
    );

    let device: &wgpu::Device = &gpu.context.device;

    let frame: LevelFrame = view_state.begin_frame((device, encoder), None)?;

    Some((id, frame, None))
  }

  /// Prepares one view of a scene for its frame: writes its camera, its jitter and motion with it, asks for the
  /// textures its weather draws with and its overlays where it has them, prepares its features and offers its periodic
  /// work; what loading and preparing take is added to `load` and `prepare`.
  fn prepare_scene_view(
    (gpu, shaders): (&mut GpuState, &ShaderLibrary),
    (scene, view_state, binding, is_scene_first): (&mut LevelScene, &mut SceneView, &ViewBinding, bool),
    framing: &SceneViewFraming<'_>,
    (encoder, periodic): (&mut FrameEncoder, &mut PeriodicScheduler),
    (load, prepare): (&mut Duration, &mut Duration),
  ) {
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
      gpu.context.dlss.is_some(),
    );
    let drawn: CameraView = view.jittered(jitter, Vec2::new(drawn_rect.width as f32, drawn_rect.height as f32));
    let unjittered: Mat4 = view.get_view_projection();
    let motion: (Mat4, Mat4) = view_state.next_motion(unjittered);
    let previous: Mat4 = view_state.next_jittered(drawn.get_view_projection());
    let selected: Option<StaticSelection> = view_state.resolve_selection(scene, selection).cloned();
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
        .with_physical_light(options)
        .with_light_view(options.mode.debug_view)
        .with_texture_bias(view_state.get_texture_bias())
        .with_motion(motion)
        .with_previous(previous)
        .with_asset_view(options, drawn_rect.height as f32 / rect.height.max(1) as f32),
    );

    let loading: Instant = Instant::now();

    {
      profiling::scope!("load");

      if let Some((overlays, version)) = overlays {
        view_state.set_overlays(scene, device, overlays, version, selection);
      }

      view_state.request_textures(scene, &mut gpu.weather_textures, (lighting, weather), options);
      Self::ask_heights(&mut gpu.textures, options);
    }
    *load += loading.elapsed();

    let preparing: Instant = Instant::now();

    {
      profiling::scope!("prepare");

      // Held apart from the scene, which preparing changes, for the weather textures to be read in its scope.
      let source: Arc<dyn RenderLevelSource> = Arc::clone(&scene.source);

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
          output: rect,
          options,
          lighting,
          weather,
          weather_textures: gpu.weather_textures.scoped(source.get_texture_scope()),
          weather_rate,
          is_scene_first,
        },
      );
      view_state.offer_periodic(scene, device, periodic, &gpu.view_layout);
    }
    *prepare += preparing.elapsed();
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

  /// Moves a scene's wetness on, once a frame, by its driving viewport's rain on the scene's clock; every view of the
  /// scene wets its surfaces by it.
  fn advance_wetness(scene: &mut LevelScene, viewports: &BTreeMap<RenderViewportId, RenderViewport>) {
    let Some(driver) = scene.driver.and_then(|driver| viewports.get(&driver)) else {
      return;
    };
    let (lighting, _): (Cow<'_, RenderLighting>, Option<&Arc<RenderLevelWeather>>) =
      Self::get_view_lighting(&driver.options, &driver.world);

    scene
      .wetness
      .advance(&lighting, &driver.options.features.rain, scene.get_time().as_secs_f32());
  }
}
