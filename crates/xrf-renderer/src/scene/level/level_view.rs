use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec2, Vec3};
use xrf_error::XrfResult;

use xrf_math::EPS_S;

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::pick_target::PickTarget;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_exposure::ViewExposure;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::view_light_groups::ViewLightGroups;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_loader::LevelLoader;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::surface_tally::SurfaceTally;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;

/// What a pick's texel says it met: a cluster, by its index and place, or an impostor, by its index.
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;

/// Sectors put on the GPU at most each frame, so a level's open spreads over frames rather than stalling one.
const SECTORS_PER_FRAME: usize = 4;

/// The engine's progressive mesh thresholds, in screen area before the screen is applied: whole above the first,
/// coarsest below the second.
const GLOD_START: f32 = 256.0;
const GLOD_END: f32 = 64.0;

/// `r_ssaLOD_A` and `r_ssaLOD_B`: a clump's impostor draws below the first, its trees above the second.
const SSA_LOD_A: f32 = 64.0;
const SSA_LOD_B: f32 = 48.0;

/// `r_ssaDISCARD`: an instanced place smaller on screen than this is not drawn.
const SSA_DISCARD: f32 = 3.5;

/// A level drawn in one viewport: read by its loader, held on the GPU, drawn into the viewport's G-buffer and lit.
pub struct LevelView {
  source: Arc<dyn RenderLevelSource>,
  loader: LevelLoader,
  scene: StaticScene,
  targets: Option<ViewTargets>,
  /// The depth pyramid over the targets, and its reduction's bind group a level.
  pyramid: Option<(DepthPyramid, Vec<wgpu::BindGroup>)>,
  /// Made again with the targets, which the cull's bind group follows.
  targets_epoch: u64,
  cull_params: wgpu::Buffer,
  occlusion: wgpu::Buffer,
  lighting: wgpu::Buffer,
  /// The view and projection the pyramid holds a frame's depth through, once one was reduced.
  history: Option<(Mat4, Mat4)>,
  /// This frame's view and projection, which become the history once its pyramid is reduced.
  frame_view: (Mat4, Mat4),
  /// This frame's camera and shadow settings, which the shadow is fitted and drawn by.
  frame_camera: CameraView,
  shadow_settings: RenderShadowSettings,
  /// The cull's and the draws' bind groups, with the scene generation (and the cull, the targets epoch) they bind.
  cull_group: Option<((u64, u64), wgpu::BindGroup)>,
  draw_groups: Option<(u64, StaticDrawGroups)>,
  /// The lighting passes' bind groups, made again with the targets, and the shadow maps' epoch they bind.
  light_groups: Option<(u64, ViewLightGroups)>,
  shadows: LevelShadows,
  lights: LevelLights,
  lights_settings: RenderLightsSettings,
  occlusion_uniform: wgpu::Buffer,
  ambient_occlusion: RenderAmbientOcclusionSettings,
  exposure: ViewExposure,
  params: StaticCullParams,
  stats: StatsReadback,
  pick_target: Option<PickTarget>,
  pick_view: Option<ViewBinding>,
  surfaces: SurfaceTally,
  failed: u32,
  reported: Option<RenderLoadReport>,
}

impl LevelView {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
  ) -> Self {
    let uniform = |label: &str, size: usize| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    let scene: StaticScene = StaticScene::new(device, queue);

    Self {
      loader: LevelLoader::start(Arc::clone(&source)),
      lights: LevelLights::new(device, view_layout, scene.args.size(), &source),
      scene,
      targets: None,
      pyramid: None,
      targets_epoch: 0,
      cull_params: uniform("static cull", size_of::<StaticCullParams>()),
      occlusion: uniform("static occlusion", size_of::<StaticOcclusionUniform>()),
      lighting: uniform("lighting", size_of::<LightingUniform>()),
      history: None,
      frame_view: (Mat4::IDENTITY, Mat4::IDENTITY),
      frame_camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      shadow_settings: RenderShadowSettings::default(),
      cull_group: None,
      draw_groups: None,
      light_groups: None,
      shadows: LevelShadows::new(device),
      lights_settings: RenderLightsSettings::default(),
      occlusion_uniform: uniform("ambient occlusion", size_of::<AmbientOcclusionUniform>()),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      exposure: ViewExposure::new(device, queue),
      params: StaticCullParams::default(),
      stats: StatsReadback::new(device),
      pick_target: None,
      pick_view: None,
      surfaces: SurfaceTally::default(),
      failed: 0,
      reported: None,
      source,
    }
  }

  /// Puts the sectors the loader finished since the last frame into the scene, a few a frame.
  pub fn load(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    textures: &mut TextureCache,
  ) {
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&self.source) as Arc<dyn RenderAssetSource>;

    self.lights.poll(textures, &assets);

    for (_, package) in self.loader.take(SECTORS_PER_FRAME) {
      match package {
        Ok((package, tally)) => {
          self.scene.add_sector(
            device,
            queue,
            encoder,
            textures,
            &assets,
            self.source.get_surfaces(),
            &package,
          );
          self.surfaces.merge(tally);
        }
        Err(_) => self.failed += 1,
      }
    }
  }

  /// How much each shader table entry draws across the sectors resident.
  pub fn measure_surfaces(&self) -> Vec<RenderSurfaceGeometry> {
    self.surfaces.list()
  }

  /// What became of every texture the level's surfaces sample.
  pub fn describe_textures(&self, textures: &TextureCache) -> Vec<RenderTextureReport> {
    textures.describe(&self.scene.texture_slots)
  }

  /// Sizes the targets to the viewport and writes what this frame's cull and lighting read.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    passes: LevelPasses<'_>,
    view: &CameraView,
    (width, height): (u32, u32),
    field_of_view: f32,
    options: &RenderViewOptions,
  ) {
    if !self.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      let targets: ViewTargets = ViewTargets::new(device, width, height);
      let pyramid: DepthPyramid = DepthPyramid::new(device, width, height);
      let groups: Vec<wgpu::BindGroup> = passes.pyramid.create_bind_groups(device, &targets.depth, &pyramid);

      self.targets = Some(targets);
      self.pyramid = Some((pyramid, groups));
      self.targets_epoch += 1;
      // A pyramid of another size holds no depth this frame can be tested against.
      self.history = None;
      self.light_groups = None;
    }

    self.scene.reset_draws(device, queue, encoder);

    let generation: u64 = self.scene.get_generation();
    let cull_key: (u64, u64) = (generation, self.targets_epoch);

    if self.cull_group.as_ref().is_none_or(|(it, _)| *it != cull_key)
      && let Some((pyramid, _)) = &self.pyramid
    {
      let group: wgpu::BindGroup = passes.cull.create_bind_group(
        device,
        &self.scene,
        &self.cull_params,
        &pyramid.view,
        &self.occlusion,
        (self.scene.lists.get_buffer(), &self.scene.args),
      );

      self.cull_group = Some((cull_key, group));
    }

    if self.draw_groups.as_ref().is_none_or(|(it, _)| *it != generation) {
      self.draw_groups = Some((generation, passes.gbuffer.create_bind_groups(device, &self.scene)));
    }

    self.shadows.prepare(device, options.shadows.resolution);

    let shadow_epoch: u64 = self.shadows.get_epoch();

    if self
      .light_groups
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != shadow_epoch)
      && let Some(targets) = &self.targets
    {
      let groups: ViewLightGroups = ViewLightGroups {
        sun: passes
          .sun
          .create_bind_group(device, targets, passes.table, &self.lighting, &self.shadows),
        lights: passes.lights.create_bind_groups(
          device,
          targets,
          passes.table,
          &self.lights.get_buffers(),
          self.lights.get_shadow_atlas(),
        ),
        occlusion: passes
          .ambient_occlusion
          .create_bind_groups(device, targets, &self.occlusion_uniform),
        combine: passes
          .combine
          .create_bind_group(device, targets, passes.table, &self.lighting, &self.exposure.state),
        exposure: passes.exposure.create_bind_group(device, targets, &self.exposure),
        present: passes.present.create_bind_group(device, targets),
      };

      self.light_groups = Some((shadow_epoch, groups));
    }

    self.exposure.prepare(queue, &options.exposure, Instant::now());

    if !options.is_occlusion_culled {
      self.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.params = StaticCullParams {
      cluster_count: self.scene.get_cluster_count(),
      row_count: self.scene.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: self.scene.get_impostor_count(),
      glod_start: threshold(GLOD_START),
      glod_end: threshold(GLOD_END),
      discard_below: SSA_DISCARD.powi(2) / screen,
      candidate_capacity: self.scene.get_list_capacity(),
      is_occluding: options.is_occlusion_culled as u32,
      lod_a: threshold(SSA_LOD_A),
      lod_b: threshold(SSA_LOD_B),
      is_impostors: options.is_impostors as u32,
      pad: [0; 4],
      lod_origin: view.position.extend(1.0),
    };
    self.frame_view = (view.view, view.projection);
    self.lights.prepare(
      queue,
      view,
      &options.lights,
      (self.params.glod_start, self.params.glod_end),
      self.scene.sectors.len(),
    );
    self.frame_camera = *view;
    self.shadow_settings = options.shadows.clone();
    self.ambient_occlusion = options.ambient_occlusion;
    self.lights_settings = options.lights;

    queue.write_buffer(
      &self.occlusion_uniform,
      0,
      bytemuck::bytes_of(&AmbientOcclusionUniform::new(
        &options.ambient_occlusion,
        view.projection,
        (width.div_ceil(2), height.div_ceil(2)),
      )),
    );
    queue.write_buffer(&self.cull_params, 0, bytemuck::bytes_of(&self.params));

    if let Some((pyramid, _)) = &self.pyramid {
      let (history_view, history_projection): (Mat4, Mat4) = self.history.unwrap_or(self.frame_view);

      queue.write_buffer(
        &self.occlusion,
        0,
        bytemuck::bytes_of(&StaticOcclusionUniform {
          view: history_view,
          projection: history_projection,
          size: Vec2::new(pyramid.width as f32, pyramid.height as f32),
          levels: pyramid.levels,
          has_history: self.history.is_some() as u32,
        }),
      );
    }
    queue.write_buffer(
      &self.lighting,
      0,
      bytemuck::bytes_of(&LightingUniform::new(
        &RenderLighting::default(),
        view.view,
        options,
        self.exposure.is_adapting(),
      )),
    );
  }

  /// Culls the scene and draws it into the G-buffer: what last frame's depth does not hide, then, culling occlusion,
  /// what this frame's first draw does not hide of the rest, leaving this frame's depth reduced for the next.
  #[allow(clippy::too_many_arguments)]
  pub fn record(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    passes: LevelPasses<'_>,
    view_layout: &wgpu::BindGroupLayout,
    view: &ViewBinding,
    textures: &TextureCache,
  ) {
    let (Some(targets), Some((pyramid, pyramid_groups)), Some((_, cull_group)), Some((_, draw_groups))) =
      (&self.targets, &self.pyramid, &self.cull_group, &self.draw_groups)
    else {
      return;
    };
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();

    passes.cull.dispatch_early(encoder, view, cull_group, &self.params);
    passes.gbuffer.draw(
      encoder,
      targets,
      view,
      draw_groups,
      texture_group,
      &self.scene.args,
      true,
    );

    if self.params.is_occluding != 0 {
      passes.pyramid.dispatch(encoder, pyramid, pyramid_groups);
      passes.cull.dispatch_late(encoder, view, cull_group, &self.scene);
      passes.gbuffer.draw(
        encoder,
        targets,
        view,
        draw_groups,
        texture_group,
        &self.scene.late,
        false,
      );
      self.history = Some(self.frame_view);
    }

    self.stats.record(encoder, &self.scene.args, StaticScene::STATS_OFFSET);

    if let Some((pyramid, _)) = &self.pyramid {
      let frame: ShadowFrame<'_> = ShadowFrame {
        scene: &self.scene,
        camera: &self.frame_camera,
        settings: &self.shadow_settings,
        sun_direction: RenderLighting::default().get_sun_direction(),
        cull_params: &self.cull_params,
        params: &self.params,
        pyramid: &pyramid.view,
        occlusion: &self.occlusion,
        targets_epoch: self.targets_epoch,
        textures,
      };

      self.shadows.record(device, queue, encoder, passes, view_layout, &frame);
      self.lights.record_shadows(device, queue, encoder, passes, &frame);
    }

    if let Some((_, groups)) = &self.light_groups {
      passes.sun.draw(encoder, targets, view, &groups.sun);

      if self.lights.get_count() > 0 {
        passes
          .lights
          .draw(encoder, targets, view, &groups.lights, textures.get_bind_group());
      }

      if self.ambient_occlusion.is_enabled {
        passes.ambient_occlusion.draw(
          encoder,
          targets,
          view,
          &groups.occlusion,
          self.ambient_occlusion.quality,
        );
      }

      passes.combine.draw(encoder, targets, view, &groups.combine);

      if self.exposure.is_adapting() {
        passes.exposure.dispatch(encoder, &groups.exposure);
      }
    }
  }

  /// Draws the frame's visible clusters into a pick's texel, through the frame's camera narrowed to it.
  #[allow(clippy::too_many_arguments)]
  pub fn record_pick(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    draw: &StaticGBufferPass,
    view_layout: &wgpu::BindGroupLayout,
    textures: &TextureCache,
    camera: &CameraUniform,
  ) {
    let Some((_, draw_groups)) = &self.draw_groups else {
      return;
    };
    let target: &PickTarget = self.pick_target.get_or_insert_with(|| PickTarget::new(device));
    let view: &ViewBinding = self
      .pick_view
      .get_or_insert_with(|| ViewBinding::new(device, view_layout));

    view.write(queue, camera);
    draw.pick(
      encoder,
      target,
      view,
      draw_groups,
      textures.get_bind_group(),
      &[&self.scene.args, &self.scene.late],
    );
  }

  /// Reads a recorded pick back, once its frame was submitted, and names what it met.
  pub fn resolve_pick(
    &self,
    device: &wgpu::Device,
    unproject: impl Fn(f32) -> Vec3,
  ) -> XrfResult<Option<RenderLevelHit>> {
    let Some(target) = &self.pick_target else {
      return Ok(None);
    };
    let [kind, cluster, place, depth] = target.read(device)?;
    let point: [f32; 3] = unproject(f32::from_bits(depth)).to_array();

    match kind {
      PICKED_CLUSTER => {}
      PICKED_IMPOSTOR => {
        return Ok(
          self
            .scene
            .resolve_impostor_pick(cluster)
            .map(|(sector, shader_id)| RenderLevelHit {
              sector,
              shader_id: shader_id as u32,
              mesh: None,
              place: None,
              is_impostor: true,
              point,
            }),
        );
      }
      _ => return Ok(None),
    }

    Ok(
      self
        .scene
        .resolve_pick(cluster, place)
        .map(|(info, instance)| RenderLevelHit {
          sector: info.sector,
          shader_id: info.shader_id as u32,
          mesh: info.mesh,
          place: instance,
          is_impostor: false,
          point,
        }),
    )
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.stats.request();
  }

  /// Clusters and triangles the latest counted frame drew.
  pub fn take_stats(&mut self) -> (u32, u32) {
    let [clusters, triangles, ..] = self.stats.take();

    (clusters, triangles)
  }

  /// What puts the level's finished scene into the window, once its targets are made.
  pub fn get_present_group(&self) -> Option<&wgpu::BindGroup> {
    self.light_groups.as_ref().map(|(_, groups)| &groups.present)
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_report(&mut self, textures: &TextureCache) -> Option<RenderLoadReport> {
    let slots = &self.scene.texture_slots;
    let (settled, total): (u32, u32) = (textures.count_settled(slots), slots.len() as u32);
    let sectors: u32 = self.scene.sectors.len() as u32 + self.failed;
    let report: RenderLoadReport = RenderLoadReport {
      sectors: self.scene.sectors.len() as u32,
      sectors_total: self.loader.get_total(),
      bytes: self.scene.get_bytes(),
      textures: settled,
      textures_total: total,
      is_ready: sectors == self.loader.get_total() && settled == total,
    };

    if self.reported == Some(report) {
      return None;
    }

    self.reported = Some(report);

    Some(report)
  }
}
