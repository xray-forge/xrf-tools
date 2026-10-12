use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, GraphTextureAccess,
  StorageArray, StorageArrayMut,
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
use crate::pass::shadow_casters::ShadowCasters;
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

/// Each cascade's passes, as the frame's timings name them, in the order a job lists them: a whole draw; its static
/// layer drawn, copied in and the trees drawn over it; and the copy with the trees over it alone.
static WHOLE_PASSES: [[&str; 1]; RENDER_MAX_SHADOW_CASCADES] =
  [["sun shadow 0"], ["sun shadow 1"], ["sun shadow 2"], ["sun shadow 3"]];
static STATIC_PASSES: [[&str; 3]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow static 0", "sun shadow copy 0", "sun shadow sway 0"],
  ["sun shadow static 1", "sun shadow copy 1", "sun shadow sway 1"],
  ["sun shadow static 2", "sun shadow copy 2", "sun shadow sway 2"],
  ["sun shadow static 3", "sun shadow copy 3", "sun shadow sway 3"],
];
static SWAY_PASSES: [[&str; 2]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow sway 0", "sun shadow copy 0"],
  ["sun shadow sway 1", "sun shadow copy 1"],
  ["sun shadow sway 2", "sun shadow copy 2"],
  ["sun shadow sway 3", "sun shadow copy 3"],
];

/// Milliseconds each cascade's whole draw costs before the timer measured one, the farther covering more; and its
/// trees over its static layer's copy.
const WHOLE_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.35, 0.45, 0.7, 1.0];
const SWAY_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.12, 0.15, 0.25, 0.35];

/// How a cascade due this frame is drawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CascadeDraw {
  /// Every caster into its map: while the trees stand still.
  Whole,
  /// What stands still into its static layer, copied into its map, the trees over it.
  Static,
  /// Its static layer copied into its map, the trees over it.
  Swaying,
}

/// A level's sun shadow: its cascades fitted along the camera's view every frame, each culled and drawn into its map
/// only when its box moved, the scene changed under it or its trees' sway shows. The nearest draws at once, and so does
/// one never drawn or whose box strayed too far from its map to wait; a farther one is offered to the frame's
/// scheduler, waiting at most its stagger (two, four, eight frames).
///
/// While the trees sway, each cascade keeps what stands still in a static layer of its own, drawn only when its box
/// moves or the scene changes under it; a sway's draw copies that layer into the map and draws the trees over it, which
/// leaves the map as a whole draw would.
#[derive(Default)]
pub struct LevelShadows {
  /// The maps, while any cascade is drawn.
  maps: Option<SunShadowMaps>,
  /// The cascades' static layers, while any cascade is drawn and the trees sway.
  statics: Option<SunShadowMaps>,
  values: ShadowUniform,
  cascades: Vec<ShadowCascadeView>,
  /// The cascades offered this frame, by their index, ticket, the version and contents a draw marks them with, and how
  /// each would be drawn.
  offers: Vec<(usize, PeriodicTicket, (u64, usize), CascadeDraw)>,
  /// The cascades this frame draws, as `commit_cascades` took them, and how.
  due: Vec<(usize, CascadeDraw)>,
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
      self.statics = None;
      self.cascades.iter_mut().for_each(|cascade| {
        cascade.drawn = None;
        cascade.drawn_static = None;
      });
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

    // The static layers pay only while the trees sway: still, a cascade draws again only where its static layer
    // would.
    let is_swaying: bool = frame.sway.amplitude * frame.sway.reach > 0.0;

    match (&self.maps, is_swaying && count > 0) {
      (Some(maps), true) if self.statics.is_none() => {
        self.statics = Some(SunShadowMaps::new_static(device, maps.resolution));
      }
      (_, false) if self.statics.is_some() => {
        self.statics = None;
        self.cascades.iter_mut().for_each(|cascade| cascade.drawn_static = None);
      }
      _ => {}
    }

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
  /// into its layer of the maps, whole, or over its static layer's copy.
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

    let statics: Option<GraphTexture> = self
      .statics
      .as_ref()
      .map(|statics| bindings.import_view(&mut *graph, "sun static shadow maps", &statics.view));
    let due: Vec<(usize, &'a ShadowCascadeView, [GraphBuffer; 2], StaticCullParameters)> = self
      .due
      .iter()
      .map(|&(index, _)| {
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

    for ((index, cascade, [args, lists], _), (_, draw)) in due.into_iter().zip(&self.due) {
      let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));
      let layer: u32 = index as u32;
      let draw_casters = |graph: &mut FrameGraph<'a>,
                          name: &'static str,
                          (target, load): (GraphTexture, wgpu::LoadOp<f32>),
                          casters: ShadowCasters| {
        StaticDraws::declare_layouts(
          graph
            .add_raster_pass(name)
            .depth(GraphDepthAttachment::new(target, load).with_array_layer(layer))
            .buffer(args, GraphBufferAccess::Indirect),
          &layouts,
        )
        .record(move |context| {
          let args: &wgpu::Buffer = context.get_buffer(args);

          passes
            .shadow
            .record_some(context, (&cascade.view, &layouts, textures), args, casters);
        });
      };

      match (*draw, statics) {
        (CascadeDraw::Static | CascadeDraw::Swaying, Some(statics)) => {
          let [static_pass, copy_pass, sway_pass] = STATIC_PASSES[index];

          if *draw == CascadeDraw::Static {
            draw_casters(
              &mut *graph,
              static_pass,
              (statics, wgpu::LoadOp::Clear(0.0)),
              ShadowCasters::Fixed,
            );
          }

          add_layer_copy(&mut *graph, copy_pass, (statics, maps), layer);
          draw_casters(
            &mut *graph,
            sway_pass,
            (maps, wgpu::LoadOp::Load),
            ShadowCasters::Swaying,
          );
        }
        _ => draw_casters(
          &mut *graph,
          WHOLE_PASSES[index][0],
          (maps, wgpu::LoadOp::Clear(0.0)),
          ShadowCasters::All,
        ),
      }
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

    for (index, ticket, state, draw) in std::mem::take(&mut self.offers) {
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

      self.due.push((index, draw));
      cascade.drawn = Some(state);

      if draw == CascadeDraw::Static {
        cascade.drawn_static = Some(state);
      }

      cascade.drawn_at = frame.sway.time;
      cascade.drawn_fit = cascade.cascade;
      cascade.waited = 0;
      self.values.matrices[index] = cascade.cascade.view.get_view_projection();
      self.values.texels[index] = cascade.cascade.texel;
    }
  }

  /// Forgets that the cascades this frame drew were drawn, where its frame was not submitted: each draws again.
  pub fn abandon_frame(&mut self) {
    for (index, _) in std::mem::take(&mut self.due) {
      self.cascades[index].drawn = None;
      self.cascades[index].drawn_static = None;
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
    let is_split: bool = self.statics.is_some();
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

      // Its static layer, where it keeps one, holds what stands still as the map would at the same version and contents.
      let is_static_stale: bool = cascade.drawn_static.is_none_or(|(version, contents)| {
        version != cascade.cascade.version
          || scene.is_changed_since(contents, |sphere| cascade.cascade.view.reaches(sphere))
      });
      let (draw, passes, seed): (CascadeDraw, &'static [&'static str], f32) = match (is_split, is_static_stale) {
        (false, _) => (CascadeDraw::Whole, &WHOLE_PASSES[index], WHOLE_SEEDS[index]),
        (true, true) => (CascadeDraw::Static, &STATIC_PASSES[index], WHOLE_SEEDS[index]),
        (true, false) => (CascadeDraw::Swaying, &SWAY_PASSES[index], SWAY_SEEDS[index]),
      };
      // The nearest is seen closest, and a cascade never drawn or strayed past its map shows wrong until drawn.
      let is_now: bool = index == 0
        || cascade.drawn.is_none()
        || !frame.settings.is_staggered
        || !cascade.cascade.is_held_by(&cascade.drawn_fit);
      let job: PeriodicJob = if is_now {
        PeriodicJob::new_now(passes, seed, 1.0)
      } else {
        PeriodicJob::new_waiting(passes, seed, 1.0, (cascade.waited, 1 << index))
      };

      self.offers.push((index, scheduler.offer(job), state, draw));
    }
  }
}

impl ViewFeature for LevelShadows {
  /// The maps and the static layers, a layer for every cascade there may be, while held.
  fn get_memory(&self) -> u64 {
    [&self.maps, &self.statics]
      .into_iter()
      .flatten()
      .map(|maps| u64::from(maps.resolution).pow(2) * RENDER_MAX_SHADOW_CASCADES as u64 * 4)
      .sum()
  }

  fn release(&mut self) {
    self.maps = None;
    self.statics = None;
    self.cascades.iter_mut().for_each(|cascade| {
      cascade.drawn = None;
      cascade.drawn_static = None;
    });
  }
}

/// Adds a pass copying one layer of a cascade's static layers into the same layer of its maps.
fn add_layer_copy(
  graph: &mut FrameGraph<'_>,
  name: &'static str,
  (statics, maps): (GraphTexture, GraphTexture),
  layer: u32,
) {
  graph
    .add_encoder_pass(name)
    .texture(statics, GraphTextureAccess::CopySource)
    .texture(maps, GraphTextureAccess::CopyDestination)
    .record(move |context| {
      let (source, destination) = (context.get_texture(statics), context.get_texture(maps));
      let size: wgpu::Extent3d = wgpu::Extent3d {
        depth_or_array_layers: 1,
        ..source.texture.size()
      };

      context.get_encoder().copy_texture_to_texture(
        to_layer(source.texture, layer),
        to_layer(destination.texture, layer),
        size,
      );
    });
}

/// One layer of a texture, whole, as a copy reads or writes it.
fn to_layer(texture: &wgpu::Texture, layer: u32) -> wgpu::TexelCopyTextureInfo<'_> {
  wgpu::TexelCopyTextureInfo {
    texture,
    mip_level: 0,
    origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
    aspect: wgpu::TextureAspect::All,
  }
}
