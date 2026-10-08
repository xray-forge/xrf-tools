use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, StorageArray,
  StorageArrayMut,
};

use crate::contract::render_rect::RenderRect;
use crate::contract::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic_job::PeriodicJob;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::lighting::sun_view_rays::SunViewRays;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::shadow_uniform::ShadowUniform;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::shadow_cascade_view::ShadowCascadeView;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::static_layout::StaticLayout;

/// Each cascade's draw, as the frame's timings name it.
static CASCADE_PASSES: [&str; RENDER_MAX_SHADOW_CASCADES] =
  ["sun shadow 0", "sun shadow 1", "sun shadow 2", "sun shadow 3"];

/// Milliseconds each cascade's draw costs before the timer measured one, the farther covering more.
const CASCADE_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.35, 0.45, 0.7, 1.0];

/// A level's sun shadow: its cascades fitted along the camera's view every frame, each culled and drawn into its map
/// only when its box moved, the scene changed under it or its trees' sway shows. The nearest draws at once, and so does
/// one never drawn or whose box strayed too far from its map to wait; a farther one is offered to the frame's
/// scheduler, waiting at most its stagger (two, four, eight frames).
#[derive(Default)]
pub struct LevelShadows {
  /// The maps, while any cascade is drawn.
  maps: Option<SunShadowMaps>,
  values: ShadowUniform,
  cascades: Vec<ShadowCascadeView>,
  /// The cascades offered this frame, by their index, ticket and the version and contents a draw marks them with.
  offers: Vec<(usize, PeriodicTicket, (u64, usize))>,
  /// The cascades this frame draws, as `commit_cascades` took them.
  due: Vec<usize>,
}

impl LevelShadows {
  /// The maps, while any cascade is drawn.
  pub fn get_maps(&self) -> Option<&SunShadowMaps> {
    self.maps.as_ref()
  }

  /// What the passes lit by the sun sample its cascades by, as this frame fitted them.
  pub fn get_values(&self) -> &ShadowUniform {
    &self.values
  }

  /// Makes the maps at the resolution the settings ask for while any cascade is drawn (`is_cast`), again where it
  /// changed, so the sun binds them before the frame draws; lets them go while none is.
  pub fn prepare(&mut self, device: &wgpu::Device, resolution: u32, is_cast: bool) {
    let resolution: u32 = resolution.clamp(1, device.limits().max_texture_dimension_2d);

    if !is_cast {
      self.release();
    } else if self.maps.as_ref().is_none_or(|maps| maps.resolution != resolution) {
      self.maps = Some(SunShadowMaps::new(device, resolution));
      self.cascades.iter_mut().for_each(|cascade| cascade.drawn = None);
    }
  }

  /// Fits the cascades, offers those wanting a draw to `scheduler`, and takes what the sun samples them by.
  pub fn prepare_cascades(
    &mut self,
    device: &wgpu::Device,
    view_layout: &wgpu::BindGroupLayout,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
  ) {
    let settings = frame.settings;
    let count: usize = settings.get_cascade_count();

    self.values.count = count as u32;
    self.offers.clear();
    self.due.clear();

    if count > 0 {
      self.offer_cascades(device, view_layout, frame, count, scheduler);
    }

    let look: Vec3 = -frame.camera.view.inverse().z_axis.truncate();

    self.values.forward = look.extend(0.0);
    self.values.filter_reach = settings.filter;
    self.values.resolution = self.maps.as_ref().map_or(1, |maps| maps.resolution) as f32;
    self.values.bias = settings.bias;
    self.values.blend = settings.blend;
  }

  /// Declares the cascades due this frame: every one culled into its own lists in one compute pass, then each drawn
  /// into its layer of the maps.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (scene, maps): (&StaticSceneHandles, GraphTexture),
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
  ) {
    if self.due.is_empty() {
      return;
    }

    let due: Vec<(usize, &'a ShadowCascadeView, [GraphBuffer; 2], StaticCullParameters)> = self
      .due
      .iter()
      .map(|&index| {
        let cascade: &'a ShadowCascadeView = &self.cascades[index];
        let args: GraphBuffer = bindings.import_buffer(graph, "sun shadow draw arguments", &cascade.args);
        let lists: GraphBuffer = bindings.import_buffer(graph, "sun shadow lists", cascade.lists.get_buffer());

        (
          index,
          cascade,
          [args, lists],
          scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args)),
        )
      })
      .collect();

    due
      .iter()
      .fold(graph.add_compute_pass("sun shadow cull"), |builder, (.., cull)| {
        builder.parameters(cull)
      })
      .record({
        let culls: Vec<(&'a ShadowCascadeView, StaticCullParameters)> =
          due.iter().map(|(_, cascade, _, cull)| (*cascade, *cull)).collect();

        move |context| {
          for (cascade, cull) in &culls {
            passes
              .cull
              .record_shadow(context, &cascade.view, cull, params, ShadowCull::Cascade);
          }
        }
      });

    for (index, cascade, [args, lists], _) in due {
      let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));

      StaticDraws::declare_layouts(
        graph
          .add_raster_pass(CASCADE_PASSES[index])
          .depth(GraphDepthAttachment::new(maps, wgpu::LoadOp::Clear(0.0)).with_array_layer(index as u32))
          .buffer(args, GraphBufferAccess::Indirect),
        &layouts,
      )
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        passes.shadow.record(context, (&cascade.view, &layouts, textures), args);
      });
    }
  }

  /// Readies the cascades `scheduler` granted to be culled and drawn, marking each drawn; one not granted waits a
  /// frame more. `encoder` takes the copies a list growing makes.
  pub fn commit_cascades(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let resolution: u32 = self.maps.as_ref().map_or(1, |maps| maps.resolution);
    let list_bytes: u64 = (scene.cull.get_list_capacity().max(1) as u64) * 8;

    for (index, ticket, state) in std::mem::take(&mut self.offers) {
      let cascade: &mut ShadowCascadeView = &mut self.cascades[index];

      if !scheduler.is_granted(ticket) {
        cascade.waited += 1;

        continue;
      }

      cascade.lists.reserve(device, encoder, list_bytes);
      scene.cull.reset_args(encoder, &cascade.args);
      cascade.view.write(
        encoder,
        &CameraUniform::new(
          &cascade.cascade.view,
          RenderRect {
            x: 0,
            y: 0,
            width: resolution,
            height: resolution,
          },
          Vec4::ZERO,
        ),
      );

      self.due.push(index);
      cascade.drawn = Some(state);
      cascade.drawn_at = frame.sway.time;
      cascade.drawn_fit = cascade.cascade;
      cascade.waited = 0;
      self.values.matrices[index] = cascade.cascade.view.get_view_projection();
      self.values.texels[index] = cascade.cascade.texel;
    }
  }

  /// Forgets that the cascades this frame drew were drawn, where its frame was not submitted: each draws again.
  pub fn abandon_frame(&mut self) {
    for index in std::mem::take(&mut self.due) {
      self.cascades[index].drawn = None;
    }
  }

  fn offer_cascades(
    &mut self,
    device: &wgpu::Device,
    view_layout: &wgpu::BindGroupLayout,
    frame: &ShadowFrame<'_>,
    count: usize,
    scheduler: &mut PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let resolution: u32 = self.maps.as_ref().map_or(1, |maps| maps.resolution);
    let mut rays: SunViewRays = SunViewRays::new(frame.camera);

    while self.cascades.len() < count.min(RENDER_MAX_SHADOW_CASCADES) {
      self
        .cascades
        .push(ShadowCascadeView::new(device, view_layout, scene.cull.args.size()));
    }

    for (index, cascade) in self.cascades.iter_mut().take(count).enumerate() {
      let width: f32 = frame.settings.cascades.get(index).copied().unwrap_or(1.0);

      cascade.cascade.fit(
        frame.camera,
        &mut rays,
        frame.sun_direction,
        width,
        resolution,
        frame.settings.reach,
      );

      let state: (u64, usize) = (cascade.cascade.version, scene.get_contents());
      // A map holds depth in the world, sampled with the matrix it was drawn with, so a still one is still exact;
      // only an item coming or going within what it covers changes it.
      let is_changed: bool = cascade.drawn.is_none_or(|(version, contents)| {
        version != cascade.cascade.version
          || scene.is_changed_since(contents, |sphere| cascade.cascade.view.reaches(sphere))
      });

      if !(is_changed
        || frame
          .sway
          .is_redrawn(frame.sway.reach, cascade.cascade.texel, cascade.drawn_at))
      {
        cascade.waited = 0;

        continue;
      }

      let passes: &'static [&'static str] = &CASCADE_PASSES[index..=index];
      // The nearest is seen closest, and a cascade never drawn or strayed past its map shows wrong until drawn.
      let is_now: bool = index == 0
        || cascade.drawn.is_none()
        || !frame.settings.is_staggered
        || !cascade.cascade.is_held_by(&cascade.drawn_fit);
      let job: PeriodicJob = if is_now {
        PeriodicJob::new_now(passes, CASCADE_SEEDS[index], 1.0)
      } else {
        PeriodicJob::new_waiting(passes, CASCADE_SEEDS[index], 1.0, (cascade.waited, 1 << index))
      };

      self.offers.push((index, scheduler.offer(job), state));
    }
  }
}

impl ViewFeature for LevelShadows {
  /// The maps, a layer for every cascade there may be, while any cascade is drawn.
  fn get_memory(&self) -> u64 {
    self.maps.as_ref().map_or(0, |maps| {
      u64::from(maps.resolution).pow(2) * RENDER_MAX_SHADOW_CASCADES as u64 * 4
    })
  }

  fn release(&mut self) {
    self.maps = None;
    self.cascades.iter_mut().for_each(|cascade| cascade.drawn = None);
  }
}
