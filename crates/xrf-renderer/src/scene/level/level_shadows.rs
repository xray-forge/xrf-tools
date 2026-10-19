use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, GraphTextureAccess,
  StorageArray, StorageArrayMut,
};

use crate::camera::camera_frustum::CameraFrustum;
use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_shadow_settings::{RENDER_MAX_SHADOW_CASCADES, RenderShadowSettings};
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::lighting::sun_cascade::SunCascade;
use crate::lighting::sun_view_rays::SunViewRays;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_casters::ShadowCasters;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::shadow_uniform::ShadowUniform;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::shadow_cascade_view::ShadowCascadeView;
use crate::scene::level::shadow_cut::ShadowCut;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_prefetch::ShadowPrefetch;
use crate::scene::level::sun_shadow_maps::SunShadowMaps;
use crate::scene::static_scene::static_layout::StaticLayout;

/// Each cascade's passes, as the frame's timings name them, in the order a job lists them: a whole draw; its static
/// layer drawn, copied in and the trees drawn over it; the copy with the trees over it alone; and a band of its next
/// static layer drawn ahead.
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
static GRASS_PASSES: [&str; RENDER_MAX_SHADOW_CASCADES] = [
  "sun shadow grass 0",
  "sun shadow grass 1",
  "sun shadow grass 2",
  "sun shadow grass 3",
];
static KEPT_PASSES: [[&str; 2]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow kept 0", "sun shadow grass 0"],
  ["sun shadow kept 1", "sun shadow grass 1"],
  ["sun shadow kept 2", "sun shadow grass 2"],
  ["sun shadow kept 3", "sun shadow grass 3"],
];
static BAND_PASSES: [[&str; 1]; RENDER_MAX_SHADOW_CASCADES] = [
  ["sun shadow band 0"],
  ["sun shadow band 1"],
  ["sun shadow band 2"],
  ["sun shadow band 3"],
];

/// Milliseconds each cascade's whole draw costs before the timer measured one, the farther covering more; and its
/// trees over its static layer's copy.
const WHOLE_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.35, 0.45, 0.7, 1.0];
const SWAY_SEEDS: [f32; RENDER_MAX_SHADOW_CASCADES] = [0.12, 0.15, 0.25, 0.35];
/// Milliseconds a kept layer's copy with the grass over it costs before the timer measured one.
const KEPT_SEED: f32 = 0.15;
/// Sway intervals a kept cascade's trees wait at most for another's redraw this frame.
const TREES_WAIT_MOST: f32 = 2.0;
/// Metres the camera moves and degrees it turns at most before trees kept only where it saw their shadows are drawn
/// again, within the widening of the frustum they were kept for.
const RECEIVED_MOVE_MOST: f32 = 4.0;
const RECEIVED_TURN_MOST: f32 = 10.0;

/// Metres the grass stands at most over the ground, which the cards' reach is sampled past.
const GRASS_STANDING: f32 = 3.0;
/// Cascades the grass's cards are drawn into, the nearest first: past them a tuft is a few texels at most, and its
/// shadow fades out over the last one's blend into the next.
pub const GRASS_CASCADES: usize = 2;
/// The share of a map's edge the sun's pass keeps a point off (`EDGE` in `common/sun_shadow.wgsl`).
const SUN_SHADOW_EDGE: f32 = 0.02;
/// Where the cards' reach is sampled for the cascades it lands in: rays a side of the screen's grid, and the shares of
/// the reach each ray is sampled at, close ones too, where the nearest cascade shades.
const GRASS_GRID: u32 = 5;
const GRASS_SHARES: [f32; 7] = [0.05, 0.1, 0.2, 0.35, 0.5, 0.75, 1.0];

/// The share of its map the nearest cascades blend into the next over at least, and how many of them do: their texels
/// are fine enough that a tuft's self-shadow differs from the next cascade's, which a band as narrow as the far ones'
/// shows as a pop where the cascade's edge sweeps across the grass with the camera.
const NEAR_BLEND: f32 = 0.2;
const NEAR_BLENDED: usize = 2;

/// The first cascade that keeps a second static layer, which its next is drawn into ahead of the turn that would stale
/// the one shown: the far ones, whose static draws cost the most.
const BANDED_FROM: usize = 2;
/// Bands a static layer drawn ahead is drawn in, at most one a frame, and frames a band waits at most.
const BANDS: u32 = 4;
const BAND_WAIT_MOST: u32 = 2;
/// The share of what a static layer may hold past which its next is drawn ahead.
const PREFETCH_HOLD: f32 = 0.5;
/// Static layers kept: one a cascade, and a second for each drawn ahead.
const STATIC_LAYERS: u32 = (2 * RENDER_MAX_SHADOW_CASCADES - BANDED_FROM) as u32;

/// How a cascade due this frame is drawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CascadeDraw {
  /// Every caster into its map: while the trees stand still.
  Whole,
  /// What stands still into its static layer, copied into its map, the trees over it.
  Static,
  /// Its static layer copied into its map, the trees over it.
  Swaying,
  /// Its kept layer copied into its map, the grass over it: the trees as last drawn, waiting their turn.
  Kept,
}

/// A cascade's draw offered this frame: which, its ticket, how it would be drawn, the fit it is drawn at, and what the
/// scene holds.
struct CascadeOffer {
  index: usize,
  ticket: PeriodicTicket,
  draw: CascadeDraw,
  fit: SunCascade,
  contents: usize,
}

/// A level's sun shadow: its cascades fitted along the camera's view every frame, each culled and drawn into its map
/// only when its box moved, the scene changed under it, its trees' sway shows or the grass casts into it.
#[derive(Default)]
pub struct LevelShadows {
  /// The maps, while any cascade is drawn.
  maps: Option<SunShadowMaps>,
  /// The cascades' static layers, while any cascade is drawn and the trees sway.
  statics: Option<SunShadowMaps>,
  /// The grass's cascades' kept layers, while the grass casts: each its static layer with the trees over it as last
  /// drawn, which the grass is drawn over every frame while the trees redraw at their own interval.
  kept: Option<SunShadowMaps>,
  values: ShadowUniform,
  cascades: Vec<ShadowCascadeView>,
  /// The cascades' draws offered this frame, and their bands drawn ahead, by cascade.
  offers: Vec<CascadeOffer>,
  band_offers: Vec<(usize, PeriodicTicket)>,
  /// The cascades this frame draws, as `commit_cascades` took them, and how; and the bands, by cascade, band and layer.
  due: Vec<(usize, CascadeDraw)>,
  bands: Vec<(usize, u32, u32)>,
  /// The cascades the grass casts into this frame: each the sun's pass samples somewhere in the cards' reach the camera
  /// sees, so the grass's shadows end where the cards fade out and not at the edge of a cascade the view turns.
  grass_cascades: [bool; RENDER_MAX_SHADOW_CASCADES],
}

impl LevelShadows {
  /// Whether a cascade due this frame draws its trees into its kept layer: the grass casts into it and the kept layers
  /// are held.
  fn is_kept_draw(&self, index: usize) -> bool {
    self.kept.is_some() && self.grass_cascades[index]
  }

  /// Metres the grass's cards need stand from the eye to shadow the cascades they are drawn into by `settings`: the
  /// last such cascade's width, past which a tuft falls in a cascade drawing none.
  pub fn get_grass_reach(settings: &RenderShadowSettings) -> Option<f32> {
    let count: usize = settings.get_cascade_count().min(GRASS_CASCADES);

    count.checked_sub(1).map(|last| settings.get_cascade_width(last))
  }

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
      self.kept = None;

      for (index, cascade) in self.cascades.iter_mut().enumerate() {
        cascade.drawn = None;
        cascade.forget_static(index);
      }
    }
  }

  /// Fits the cascades, offers those wanting a draw to `scheduler` while the sun lights anything, and takes what the
  /// sun samples them by.
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

    // A cascade past those asked for holds nothing, drawn afresh once it is asked for again.
    for (index, cascade) in self.cascades.iter_mut().enumerate().skip(count) {
      cascade.release(index);
    }

    self.offers.clear();
    self.band_offers.clear();
    self.due.clear();
    self.bands.clear();

    self.grass_cascades = [false; RENDER_MAX_SHADOW_CASCADES];
    self.values.blends = Self::get_cascade_blends(settings.blend);

    // The static layers pay only while the trees sway or the grass casts: still, a cascade draws again only where its
    // static layer would.
    let is_moving: bool = frame.sway.amplitude * frame.sway.reach > 0.0 || frame.grass_reach.is_some();

    match (&self.maps, is_moving && count > 0) {
      (Some(maps), true) if self.statics.is_none() => {
        self.statics = Some(SunShadowMaps::new_static(device, maps.resolution, STATIC_LAYERS));
      }
      (_, false) if self.statics.is_some() => {
        self.statics = None;

        for (index, cascade) in self.cascades.iter_mut().enumerate() {
          cascade.forget_static(index);
        }
      }
      _ => {}
    }

    match (&self.maps, self.statics.is_some() && frame.grass_reach.is_some()) {
      (Some(maps), true) if self.kept.is_none() => {
        self.kept = Some(SunShadowMaps::new_kept(device, maps.resolution, GRASS_CASCADES as u32));
      }
      (_, false) if self.kept.is_some() => {
        self.kept = None;

        for cascade in &mut self.cascades {
          cascade.is_kept = false;
        }
      }
      _ => {}
    }

    // A sun of no colour lights nothing its shadow could take away: no cascade is drawn, the maps keeping their last.
    if count > 0 && frame.is_sunlit {
      self.offer_cascades(device, view_layout, frame, count, scheduler);
    }

    let look: Vec3 = -frame.camera.view.inverse().z_axis.truncate();

    self.values.forward = look.extend(0.0);
    self.values.filter_reach = settings.get_filter();
    self.values.resolution = self.maps.as_ref().map_or(1, |maps| maps.resolution) as f32;
    self.values.bias = settings.bias;
  }

  /// Declares the cascades due this frame and the bands drawn ahead: every one culled into its own lists in one compute
  /// pass, then each cascade drawn into its layer of the maps, whole or over its static layer's copy with the `grass`
  /// over the nearest, and each band into its cascade's hidden static layer.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (scene, maps): (&StaticSceneHandles, GraphTexture),
    (params, textures): (&'a StaticCullUniform, &'a wgpu::BindGroup),
    grass: Option<GrassShadowCasters<'a>>,
  ) {
    if self.due.is_empty() && self.bands.is_empty() {
      return;
    }

    let statics: Option<GraphTexture> = self
      .statics
      .as_ref()
      .map(|statics| bindings.import_view(&mut *graph, "sun static shadow maps", &statics.view));
    let kept: Option<GraphTexture> = self
      .kept
      .as_ref()
      .map(|kept| bindings.import_view(&mut *graph, "sun kept shadow maps", &kept.view));
    let mut import = |cut: &'a ShadowCut| -> (&'a ShadowCut, GraphBuffer, StaticCullParameters, StaticLayouts) {
      let args: GraphBuffer = bindings.import_buffer(graph, "sun shadow draw arguments", &cut.args);
      let lists: GraphBuffer = bindings.import_buffer(graph, "sun shadow lists", cut.get_lists());

      (
        cut,
        args,
        scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args)),
        scene.get_layout_draws(StorageArray::new(lists)),
      )
    };
    let due: Vec<(usize, CascadeDraw, CutHandles<'a>)> = self
      .due
      .iter()
      .map(|&(index, draw)| (index, draw, import(&self.cascades[index].cut)))
      .collect();
    let bands: Vec<(usize, u32, u32, CutHandles<'a>)> = self
      .bands
      .iter()
      .filter_map(|&(index, band, layer)| Some((index, band, layer, import(self.cascades[index].band.as_ref()?))))
      .collect();
    // A kept layer's copy draws no caster, so its cascade needs no lists; one whose trees go into its kept layer keeps
    // them only where their shadows can fall in the camera's view, the layer drawn again once the camera turns away.
    let culls: Vec<(&'a ViewBinding, StaticCullParameters, ShadowCull)> = due
      .iter()
      .filter(|(_, draw, _)| *draw != CascadeDraw::Kept)
      .map(|(index, _, (cut, _, cull, _))| {
        let kind: ShadowCull = if self.is_kept_draw(*index) {
          ShadowCull::CascadeReceived
        } else {
          ShadowCull::Cascade
        };

        (&cut.view, *cull, kind)
      })
      .chain(
        bands
          .iter()
          .map(|(.., (cut, _, cull, _))| (&cut.view, *cull, ShadowCull::Cascade)),
      )
      .collect();

    culls
      .iter()
      .fold(graph.add_compute_pass("sun shadow cull"), |builder, (_, cull, _)| {
        builder.parameters(cull)
      })
      .record(move |context| {
        for (view, cull, kind) in &culls {
          passes.cull.record_shadow(context, view, cull, params, *kind);
        }
      });

    for (index, draw, (cut, args, _, layouts)) in due {
      let layer: u32 = index as u32;
      let static_layer: u32 = self.cascades[index].static_layer;
      let draw_casters = |graph: &mut FrameGraph<'a>,
                          name: &'static str,
                          (target, layer, load): (GraphTexture, u32, wgpu::LoadOp<f32>),
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
            .record_some(context, (&cut.view, &layouts, textures), args, casters);
        });
      };

      let has_grass: bool = self.grass_cascades[index];

      match (draw, statics, kept) {
        (CascadeDraw::Kept, _, Some(kept)) => {
          Self::add_layer_copy(&mut *graph, KEPT_PASSES[index][0], (kept, maps), (layer, layer));

          if let Some(grass) = grass.filter(|_| has_grass) {
            grass.add_pass(&mut *graph, GRASS_PASSES[index], (maps, layer), &cut.view);
          }
        }
        (CascadeDraw::Static | CascadeDraw::Swaying, Some(statics), kept) => {
          let [static_pass, copy_pass, sway_pass] = STATIC_PASSES[index];
          // Where the grass casts, the trees go into the kept layer, which later frames copy in under the grass alone.
          let kept: Option<GraphTexture> = kept.filter(|_| has_grass);
          let (target, target_layer): (GraphTexture, u32) = (kept.unwrap_or(maps), layer);

          if draw == CascadeDraw::Static {
            draw_casters(
              &mut *graph,
              static_pass,
              (statics, static_layer, wgpu::LoadOp::Clear(0.0)),
              ShadowCasters::Fixed,
            );
          }

          Self::add_layer_copy(&mut *graph, copy_pass, (statics, target), (static_layer, target_layer));
          draw_casters(
            &mut *graph,
            sway_pass,
            (target, target_layer, wgpu::LoadOp::Load),
            ShadowCasters::Swaying,
          );

          if let Some(kept) = kept {
            Self::add_layer_copy(&mut *graph, KEPT_PASSES[index][0], (kept, maps), (target_layer, layer));
          }

          if let Some(grass) = grass.filter(|_| has_grass) {
            grass.add_pass(&mut *graph, GRASS_PASSES[index], (maps, layer), &cut.view);
          }
        }
        _ => draw_casters(
          &mut *graph,
          WHOLE_PASSES[index][0],
          (maps, layer, wgpu::LoadOp::Clear(0.0)),
          ShadowCasters::All,
        ),
      }
    }

    let (Some(statics), Some(resolution)) = (statics, self.maps.as_ref().map(|maps| maps.resolution)) else {
      return;
    };

    for (index, band, layer, (cut, args, _, layouts)) in bands {
      StaticDraws::declare_layouts(
        graph
          .add_raster_pass(BAND_PASSES[index][0])
          .depth(GraphDepthAttachment::new(statics, wgpu::LoadOp::Load).with_array_layer(layer))
          .buffer(args, GraphBufferAccess::Indirect),
        &layouts,
      )
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        passes.shadow.record_tile(
          context,
          (Self::get_band_region(band, resolution), None),
          (&cut.view, &layouts, textures),
          (args, 0),
          ShadowCasters::Fixed,
        );
      });
    }
  }

  /// Readies the cascades and bands `scheduler` granted to be culled and drawn, marking each drawn; one not granted
  /// waits a frame more. `encoder` takes what readying them writes.
  pub fn commit_cascades(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let resolution: u32 = self.maps.as_ref().map_or(1, |maps| maps.resolution);
    let list_bytes: u64 = scene.cull.get_list_bytes();
    let ready = |cut: &mut ShadowCut, encoder: &mut FrameEncoder, view: &CameraView, rect| {
      cut.reserve_lists(device, list_bytes);
      scene.cull.reset_args(encoder, &cut.args);
      cut.view.write(encoder, &CameraUniform::new(view, rect, Vec4::ZERO));
    };

    for offer in std::mem::take(&mut self.offers) {
      let cascade: &mut ShadowCascadeView = &mut self.cascades[offer.index];

      if !scheduler.is_granted(offer.ticket) {
        cascade.waited += 1;

        continue;
      }

      ready(
        &mut cascade.cut,
        encoder,
        &offer.fit.view,
        RenderRect {
          x: 0,
          y: 0,
          width: resolution,
          height: resolution,
        },
      );

      self.due.push((offer.index, offer.draw));
      cascade.drawn = Some((offer.fit.version, offer.contents));
      // A kept layer's copy shows the trees as last drawn, so their interval runs on from that draw.
      if offer.draw != CascadeDraw::Kept {
        cascade.drawn_at = frame.sway.time;
      }

      cascade.drawn_fit = offer.fit;
      cascade.waited = 0;
      cascade.is_swapped = false;
      cascade.has_drawn_grass = offer.draw != CascadeDraw::Whole && self.grass_cascades[offer.index];
      cascade.is_kept = offer.draw != CascadeDraw::Whole && self.kept.is_some() && self.grass_cascades[offer.index];

      if matches!(offer.draw, CascadeDraw::Static | CascadeDraw::Swaying) {
        cascade.received_at = cascade
          .is_kept
          .then(|| (frame.camera.position, frame.camera.get_look()));
      } else if offer.draw == CascadeDraw::Whole {
        cascade.received_at = None;
      }

      if offer.draw == CascadeDraw::Static {
        cascade.drawn_static = Some((offer.fit.version, offer.contents));
        cascade.static_fit = offer.fit;
        cascade.prefetch = None;
      }

      self.values.matrices[offer.index] = offer.fit.view.get_view_projection();
      self.values.texels[offer.index] = offer.fit.texel;
    }

    for (index, ticket) in std::mem::take(&mut self.band_offers) {
      let cascade: &mut ShadowCascadeView = &mut self.cascades[index];
      let (Some(prefetch), Some(cut)) = (cascade.prefetch.as_mut(), cascade.band.as_mut()) else {
        continue;
      };

      if !scheduler.is_granted(ticket) {
        prefetch.waited += 1;

        continue;
      }

      let [x, y, width, height] = Self::get_band_region(prefetch.bands, resolution);

      ready(
        cut,
        encoder,
        &prefetch.fit.get_rows_view((y, y + height), resolution),
        RenderRect {
          x: x as i32,
          y: y as i32,
          width,
          height,
        },
      );

      self.bands.push((
        index,
        prefetch.bands,
        Self::get_hidden_layer(index, cascade.static_layer),
      ));
      prefetch.bands += 1;
      prefetch.waited = 0;
    }
  }

  /// Forgets that the cascades this frame drew were drawn, where its frame was not submitted: each draws again, and a
  /// static layer drawn ahead starts again.
  pub fn abandon_frame(&mut self) {
    for (index, _) in std::mem::take(&mut self.due) {
      self.cascades[index].drawn = None;
      self.cascades[index].drawn_static = None;
      self.cascades[index].is_kept = false;
      self.cascades[index].received_at = None;
    }

    for (index, ..) in std::mem::take(&mut self.bands) {
      self.cascades[index].prefetch = None;
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
    let blends: Vec4 = self.values.blends;
    let mut rays: SunViewRays = SunViewRays::new(frame.camera);
    // How far along the light the level stands, which every cascade's casters and receivers stand within.
    let span: Option<(f32, f32)> = scene.get_span(frame.sun_direction.normalize_or_zero());

    while self.cascades.len() < count.min(RENDER_MAX_SHADOW_CASCADES) {
      let index: usize = self.cascades.len();

      self.cascades.push(ShadowCascadeView::new(device, view_layout, index));
    }

    // The points of the cards' reach the camera sees, and the cascade each is shaded by so far and whether it blends
    // into the next, as the sun's pass chooses them (`sun_shadow`): the cascades tile the view, the far ones starting
    // where the near ones end, so each shades a part of it.
    let mut points: Vec<(Vec3, Option<(usize, bool)>)> = frame
      .grass_reach
      .map_or_else(Vec::new, |reach| {
        Self::sample_seen_grass(frame.camera, reach + GRASS_STANDING)
      })
      .into_iter()
      .map(|point| (point, None))
      .collect();
    // Whether a kept cascade already redraws its trees this frame: the others wait theirs, so the redraws spread.
    let mut is_trees_taken: bool = false;

    for (index, cascade) in self.cascades.iter_mut().take(count).enumerate() {
      let width: f32 = frame.settings.get_cascade_width(index);

      cascade
        .cascade
        .fit(frame.camera, &mut rays, frame.sun_direction, width, resolution, span);

      if !is_split {
        self.offers.extend(Self::offer_whole(index, cascade, frame, scheduler));

        continue;
      }

      let projection: Mat4 = cascade.cascade.view.get_view_projection();
      let has_next: bool = index + 1 < count;
      let mut has_grass: bool = false;
      let is_grass_drawn: bool = index < GRASS_CASCADES;

      for (point, shaded) in &mut points {
        match (*shaded, Self::get_cascade_inset(projection, *point)) {
          (None, Some(inset)) => {
            has_grass = is_grass_drawn;
            *shaded = Some((index, has_next && inset < blends[index]));
          }
          (Some((at, true)), Some(_)) if at + 1 == index => has_grass = is_grass_drawn,
          _ => {}
        }
      }

      self.grass_cascades[index] = has_grass;
      Self::swap_prefetched(index, cascade);
      self.offers.extend(Self::offer_split(
        index,
        cascade,
        frame,
        (has_grass, &mut is_trees_taken),
        scheduler,
      ));

      if index >= BANDED_FROM {
        cascade
          .band
          .get_or_insert_with(|| ShadowCut::new(device, view_layout, "sun shadow band"));

        let ticket: Option<PeriodicTicket> = Self::offer_band(index, cascade, frame, scheduler);

        self.band_offers.extend(ticket.map(|ticket| (index, ticket)));
      }
    }
  }

  /// Offers the draw a cascade drawn whole wants this frame, while the trees stand still: where its box moved, the
  /// scene changed under it or the sway shows.
  fn offer_whole(
    index: usize,
    cascade: &mut ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
  ) -> Option<CascadeOffer> {
    let scene = frame.scene;
    let fit: SunCascade = cascade.cascade;
    // A map holds depth in the world, sampled with the matrix it was drawn with, so a still one is still exact; only an
    // item coming or going within what it covers changes it.
    let is_changed: bool = cascade.has_drawn_grass
      || cascade.drawn.is_none_or(|(version, contents)| {
        let frustum: CameraFrustum = fit.view.get_frustum();

        version != fit.version || scene.is_changed_since(contents, |sphere| frustum.reaches(sphere))
      });

    let interval: f32 = frame.settings.get_sway_interval();

    if !(is_changed
      || frame.settings.is_swaying(index)
        && frame
          .sway
          .is_redrawn_every(frame.sway.reach, fit.texel, cascade.drawn_at, interval))
    {
      cascade.waited = 0;

      return None;
    }

    Some(Self::offer(
      index,
      cascade,
      frame,
      scheduler,
      (CascadeDraw::Whole, fit),
      cascade.has_drawn_grass,
    ))
  }

  /// Offers the draw a cascade keeping a static layer wants this frame, while the trees sway or the grass casts: over
  /// its static layer, at the fit that layer was drawn at, while the cascade's fit holds by it and nothing changed
  /// under it (the nearest cascade's only while its fit is the same); else a new static layer at the cascade's fit. One
  /// the grass casts into (`has_grass`) draws every frame, at once: the grass sways every frame, and a shadow drawn at
  /// the sway's interval would trail the blades drawn over it wherever the frame rate passes it. Its trees still wait
  /// their interval: between their draws it copies in its kept layer and draws the grass alone over it, and while
  /// another kept cascade redraws its trees this frame (`is_trees_taken`) one whose trees are due for the sway alone
  /// waits a frame, up to `TREES_WAIT_MOST` intervals, so the redraws spread over frames rather than piling into one.
  fn offer_split(
    index: usize,
    cascade: &mut ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    (has_grass, is_trees_taken): (bool, &mut bool),
    scheduler: &mut PeriodicScheduler,
  ) -> Option<CascadeOffer> {
    let scene = frame.scene;
    let current: SunCascade = cascade.cascade;
    let static_fit: SunCascade = cascade.static_fit;
    let is_held: bool = cascade.drawn_static.is_some_and(|(version, contents)| {
      let is_fit: bool = if index == 0 {
        version == current.version
      } else {
        current.is_held_by(&static_fit)
      };

      let frustum: CameraFrustum = static_fit.view.get_frustum();

      is_fit && !scene.is_changed_since(contents, |sphere| frustum.reaches(sphere))
    });
    // Trees kept only where the camera saw their shadows are drawn again once it turned or moved far from where it was.
    let is_left: bool = cascade.received_at.is_some_and(|(eye, look)| {
      eye.distance(frame.camera.position) > RECEIVED_MOVE_MOST
        || look.dot(frame.camera.get_look()) < RECEIVED_TURN_MOST.to_radians().cos()
    });
    let is_stale: bool = cascade.drawn.is_none() || !is_held || cascade.is_swapped || is_left;
    let interval: f32 = frame.settings.get_sway_interval();
    let is_swayed: bool = frame.settings.is_swaying(index)
      && frame
        .sway
        .is_redrawn_every(frame.sway.reach, current.texel, cascade.drawn_at, interval);
    let is_kept: bool = has_grass && cascade.is_kept;
    let can_wait: bool = frame.sway.time - cascade.drawn_at < interval * TREES_WAIT_MOST;
    let is_trees_due: bool = is_stale || (is_swayed && !(is_kept && *is_trees_taken && can_wait));

    if is_kept && is_trees_due {
      *is_trees_taken = true;
    }

    if is_kept && !is_trees_due {
      return Some(Self::offer(
        index,
        cascade,
        frame,
        scheduler,
        (CascadeDraw::Kept, cascade.drawn_fit),
        true,
      ));
    }

    let is_wanted: bool = is_trees_due || has_grass || cascade.has_drawn_grass;

    if !is_wanted {
      cascade.waited = 0;

      return None;
    }

    let draw: (CascadeDraw, SunCascade) = if is_held {
      (CascadeDraw::Swaying, static_fit)
    } else {
      (CascadeDraw::Static, current)
    };

    Some(Self::offer(
      index,
      cascade,
      frame,
      scheduler,
      draw,
      has_grass || cascade.has_drawn_grass,
    ))
  }

  /// Offers a cascade's draw `draw` at a fit: at once for the nearest, one never drawn, staggering off, one whose box
  /// strayed past its map, or one the grass casts into (`is_urgent`); else within its stagger.
  fn offer(
    index: usize,
    cascade: &ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
    (draw, fit): (CascadeDraw, SunCascade),
    is_urgent: bool,
  ) -> CascadeOffer {
    let (passes, seed): (&'static [&'static str], f32) = match draw {
      CascadeDraw::Whole => (&WHOLE_PASSES[index], WHOLE_SEEDS[index]),
      CascadeDraw::Static => (&STATIC_PASSES[index], WHOLE_SEEDS[index]),
      CascadeDraw::Swaying => (&SWAY_PASSES[index], SWAY_SEEDS[index]),
      CascadeDraw::Kept => (&KEPT_PASSES[index], KEPT_SEED),
    };
    // The nearest is seen closest, and a cascade never drawn or strayed past its map shows wrong until drawn.
    let is_now: bool = index == 0
      || is_urgent
      || cascade.drawn.is_none()
      || !frame.settings.is_staggered
      || !cascade.cascade.is_held_by(&cascade.drawn_fit);
    let job: PeriodicJob = if is_now {
      PeriodicJob::new_now(passes, seed, 1.0)
    } else {
      PeriodicJob::new_waiting(passes, seed, 1.0, (cascade.waited, 1 << index))
    };

    CascadeOffer {
      index,
      ticket: scheduler.offer(job),
      draw,
      fit,
      contents: frame.scene.get_contents(),
    }
  }

  /// How far in from its map's edge a cascade drawn by `projection` holds a point, past the edge the sun's pass keeps
  /// off (`EDGE` in `common/sun_shadow.wgsl`), as a share of the map; none where it does not hold it.
  fn get_cascade_inset(projection: Mat4, point: Vec3) -> Option<f32> {
    let clip: Vec4 = projection * point.extend(1.0);
    let (u, v): (f32, f32) = (clip.x * 0.5 + 0.5, clip.y * -0.5 + 0.5);
    let inset: f32 = u.min(v).min(1.0 - u).min(1.0 - v) - SUN_SHADOW_EDGE;

    (inset > 0.0 && clip.z > 0.0 && clip.z < 1.0).then_some(inset)
  }

  /// Each cascade's blend band, the settings' `blend` widened to `NEAR_BLEND` for the nearest.
  pub(crate) fn get_cascade_blends(blend: f32) -> Vec4 {
    Vec4::from_array(std::array::from_fn(|index| {
      if index < NEAR_BLENDED {
        blend.max(NEAR_BLEND)
      } else {
        blend
      }
    }))
  }

  /// Points of the sphere `reach` metres about the `camera` that it sees, where the grass's shadows land: along rays
  /// through a grid over the screen, its edges included, at shares of the reach, so a view of any width samples it.
  /// What the camera does not see casts onto nothing it shows, and no cascade's box reaches behind it.
  pub(crate) fn sample_seen_grass(camera: &CameraView, reach: f32) -> Vec<Vec3> {
    let inverse: Mat4 = camera.get_view_projection().inverse();
    let last: f32 = (GRASS_GRID - 1) as f32;

    (0..GRASS_GRID * GRASS_GRID)
      .filter_map(|cell| {
        let x: f32 = (cell % GRASS_GRID) as f32 / last * 2.0 - 1.0;
        let y: f32 = (cell / GRASS_GRID) as f32 / last * 2.0 - 1.0;

        (inverse.project_point3(Vec3::new(x, y, 0.5)) - camera.position).try_normalize()
      })
      .flat_map(|ray| GRASS_SHARES.map(|share| camera.position + ray * reach * share))
      .collect()
  }

  /// Shows a cascade's static layer drawn ahead in place of the one shown, once every band of it is drawn.
  fn swap_prefetched(index: usize, cascade: &mut ShadowCascadeView) {
    let Some(prefetch) = cascade.prefetch.filter(|prefetch| prefetch.bands == BANDS) else {
      return;
    };

    cascade.static_layer = Self::get_hidden_layer(index, cascade.static_layer);
    cascade.static_fit = prefetch.fit;
    cascade.drawn_static = Some((prefetch.fit.version, prefetch.contents));
    cascade.prefetch = None;
    cascade.is_swapped = true;
  }

  /// Offers a band of a far cascade's next static layer, drawn ahead once its fit strayed past `PREFETCH_HOLD` of what
  /// its static layer holds, at the fit it strayed to; started again where the fit strays as far from that or the scene
  /// changes under it. None while its static layer holds well, or is drawn again whole this frame.
  fn offer_band(
    index: usize,
    cascade: &mut ShadowCascadeView,
    frame: &ShadowFrame<'_>,
    scheduler: &mut PeriodicScheduler,
  ) -> Option<PeriodicTicket> {
    let scene = frame.scene;
    let current: SunCascade = cascade.cascade;
    let is_static_drawn: bool = cascade.drawn_static.is_some() && current.is_held_by(&cascade.static_fit);

    if !is_static_drawn || current.is_held_within(&cascade.static_fit, PREFETCH_HOLD) {
      cascade.prefetch = None;

      return None;
    }

    let started: ShadowPrefetch = ShadowPrefetch {
      fit: current,
      contents: scene.get_contents(),
      bands: 0,
      waited: 0,
    };
    let prefetch: &mut ShadowPrefetch = cascade.prefetch.get_or_insert(started);
    let frustum: CameraFrustum = prefetch.fit.view.get_frustum();

    if !current.is_held_within(&prefetch.fit, PREFETCH_HOLD)
      || scene.is_changed_since(prefetch.contents, |sphere| frustum.reaches(sphere))
    {
      *prefetch = ShadowPrefetch {
        waited: prefetch.waited,
        ..started
      };
    }

    let job: PeriodicJob = PeriodicJob::new_waiting(
      &BAND_PASSES[index],
      WHOLE_SEEDS[index] / BANDS as f32,
      1.0,
      (prefetch.waited, BAND_WAIT_MOST),
    );

    Some(scheduler.offer(job))
  }

  /// The static layer a far cascade draws ahead into, the one of its two it does not show.
  fn get_hidden_layer(index: usize, shown: u32) -> u32 {
    let second: u32 = (RENDER_MAX_SHADOW_CASCADES + index - BANDED_FROM) as u32;

    if shown == index as u32 { second } else { index as u32 }
  }

  /// A band's region of a static layer `resolution` texels across: every column of its rows, the bands together
  /// covering every row.
  fn get_band_region(band: u32, resolution: u32) -> [u32; 4] {
    let (first, last): (u32, u32) = (band * resolution / BANDS, (band + 1) * resolution / BANDS);

    [0, first, resolution, last - first]
  }

  /// Adds a pass copying one layer of a cascade's static layers into its layer of the maps.
  fn add_layer_copy(
    graph: &mut FrameGraph<'_>,
    name: &'static str,
    (statics, maps): (GraphTexture, GraphTexture),
    (source, destination): (u32, u32),
  ) {
    graph
      .add_encoder_pass(name)
      .texture(statics, GraphTextureAccess::CopySource)
      .texture(maps, GraphTextureAccess::CopyDestination)
      .record(move |context| {
        let (from, into) = (context.get_texture(statics), context.get_texture(maps));
        let size: wgpu::Extent3d = wgpu::Extent3d {
          depth_or_array_layers: 1,
          ..into.texture.size()
        };

        context.get_encoder().copy_texture_to_texture(
          Self::to_layer(from.texture, source),
          Self::to_layer(into.texture, destination),
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
}

/// A cut's cull and draw as a frame's graph knows them.
type CutHandles<'a> = (&'a ShadowCut, GraphBuffer, StaticCullParameters, StaticLayouts);

/// What a cut draws, layout by layout.
type StaticLayouts = [StaticDrawParameters; StaticLayout::COUNT];

impl ViewFeature for LevelShadows {
  /// The maps and the static layers, and the cascades' lists, while held.
  fn get_memory(&self) -> u64 {
    [&self.maps, &self.statics, &self.kept]
      .into_iter()
      .flatten()
      .map(|maps| u64::from(maps.resolution).pow(2) * maps.layers.len() as u64 * 4)
      .chain(self.cascades.iter().map(ShadowCascadeView::get_memory))
      .sum()
  }

  fn release(&mut self) {
    self.maps = None;
    self.statics = None;
    self.kept = None;

    for (index, cascade) in self.cascades.iter_mut().enumerate() {
      cascade.release(index);
    }
  }
}
