use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, GraphTextureAccess,
  StorageArray, StorageArrayMut,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic_job::PeriodicJob;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::lighting::sun_cascade::SunCascade;
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
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::shadow_cascade_view::ShadowCascadeView;
use crate::scene::level::shadow_cut::ShadowCut;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_prefetch::ShadowPrefetch;
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
/// only when its box moved, the scene changed under it or its trees' sway shows. The nearest draws at once, and so does
/// one never drawn or whose box strayed too far from its map to wait; a farther one is offered to the frame's
/// scheduler, waiting at most its stagger (two, four, eight frames).
///
/// While the trees sway, each cascade keeps what stands still in a static layer of its own, drawn only when the scene
/// changes under it or its fit no longer holds by the one it was drawn at; a draw for the sway copies that layer into
/// the map and draws the trees over it at the same fit, which leaves the map as a whole draw at that fit would. The far
/// cascades draw their next static layer ahead, a band a frame into a second layer, once their fit has strayed half of
/// what their static layer holds, and show it when done: a turning sun never stales one whole on a single frame.
#[derive(Default)]
pub struct LevelShadows {
  /// The maps, while any cascade is drawn.
  maps: Option<SunShadowMaps>,
  /// The cascades' static layers, while any cascade is drawn and the trees sway.
  statics: Option<SunShadowMaps>,
  values: ShadowUniform,
  cascades: Vec<ShadowCascadeView>,
  /// The cascades' draws offered this frame, and their bands drawn ahead, by cascade.
  offers: Vec<CascadeOffer>,
  band_offers: Vec<(usize, PeriodicTicket)>,
  /// The cascades this frame draws, as `commit_cascades` took them, and how; and the bands, by cascade, band and layer.
  due: Vec<(usize, CascadeDraw)>,
  bands: Vec<(usize, u32, u32)>,
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

      for (index, cascade) in self.cascades.iter_mut().enumerate() {
        cascade.drawn = None;
        cascade.forget_static(index);
      }
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
    self.band_offers.clear();
    self.due.clear();
    self.bands.clear();

    // The static layers pay only while the trees sway: still, a cascade draws again only where its static layer
    // would.
    let is_swaying: bool = frame.sway.amplitude * frame.sway.reach > 0.0;

    match (&self.maps, is_swaying && count > 0) {
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

  /// Declares the cascades due this frame and the bands drawn ahead: every one culled into its own lists in one compute
  /// pass, then each cascade drawn into its layer of the maps, whole or over its static layer's copy, and each band
  /// into its cascade's hidden static layer.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (scene, maps): (&StaticSceneHandles, GraphTexture),
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
  ) {
    if self.due.is_empty() && self.bands.is_empty() {
      return;
    }

    let statics: Option<GraphTexture> = self
      .statics
      .as_ref()
      .map(|statics| bindings.import_view(&mut *graph, "sun static shadow maps", &statics.view));
    let mut import = |cut: &'a ShadowCut| -> (&'a ShadowCut, GraphBuffer, StaticCullParameters, StaticLayouts) {
      let args: GraphBuffer = bindings.import_buffer(graph, "sun shadow draw arguments", &cut.args);
      let lists: GraphBuffer = bindings.import_buffer(graph, "sun shadow lists", cut.lists.get_buffer());

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
    let culls: Vec<(&'a ViewBinding, StaticCullParameters)> = due
      .iter()
      .map(|(.., (cut, _, cull, _))| (&cut.view, *cull))
      .chain(bands.iter().map(|(.., (cut, _, cull, _))| (&cut.view, *cull)))
      .collect();

    culls
      .iter()
      .fold(graph.add_compute_pass("sun shadow cull"), |builder, (_, cull)| {
        builder.parameters(cull)
      })
      .record(move |context| {
        for (view, cull) in &culls {
          passes
            .cull
            .record_shadow(context, view, cull, params, ShadowCull::Cascade);
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

      match (draw, statics) {
        (CascadeDraw::Static | CascadeDraw::Swaying, Some(statics)) => {
          let [static_pass, copy_pass, sway_pass] = STATIC_PASSES[index];

          if draw == CascadeDraw::Static {
            draw_casters(
              &mut *graph,
              static_pass,
              (statics, static_layer, wgpu::LoadOp::Clear(0.0)),
              ShadowCasters::Fixed,
            );
          }

          add_layer_copy(&mut *graph, copy_pass, (statics, maps), (static_layer, layer));
          draw_casters(
            &mut *graph,
            sway_pass,
            (maps, layer, wgpu::LoadOp::Load),
            ShadowCasters::Swaying,
          );
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
          (get_band_region(band, resolution), None),
          (&cut.view, &layouts, textures),
          (args, 0),
          ShadowCasters::Fixed,
        );
      });
    }
  }

  /// Readies the cascades and bands `scheduler` granted to be culled and drawn, marking each drawn; one not granted
  /// waits a frame more. `encoder` takes the copies a list growing makes.
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
    let ready = |cut: &mut ShadowCut, encoder: &mut FrameEncoder, view: &CameraView, rect| {
      cut.lists.reserve(device, encoder, list_bytes);
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
      cascade.drawn_at = frame.sway.time;
      cascade.drawn_fit = offer.fit;
      cascade.waited = 0;
      cascade.is_swapped = false;

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

      let [x, y, width, height] = get_band_region(prefetch.bands, resolution);

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

      self
        .bands
        .push((index, prefetch.bands, get_hidden_layer(index, cascade.static_layer)));
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
    let mut rays: SunViewRays = SunViewRays::new(frame.camera);

    while self.cascades.len() < count.min(RENDER_MAX_SHADOW_CASCADES) {
      let index: usize = self.cascades.len();

      self.cascades.push(ShadowCascadeView::new(
        device,
        view_layout,
        (index, scene.cull.args.size()),
      ));
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

      if !is_split {
        self.offers.extend(offer_whole(index, cascade, frame, scheduler));

        continue;
      }

      swap_prefetched(index, cascade);
      self.offers.extend(offer_split(index, cascade, frame, scheduler));

      if index >= BANDED_FROM {
        cascade
          .band
          .get_or_insert_with(|| ShadowCut::new(device, view_layout, "sun shadow band", scene.cull.args.size()));

        let ticket: Option<PeriodicTicket> = offer_band(index, cascade, frame, scheduler);

        self.band_offers.extend(ticket.map(|ticket| (index, ticket)));
      }
    }
  }
}

/// A cut's cull and draw as a frame's graph knows them.
type CutHandles<'a> = (&'a ShadowCut, GraphBuffer, StaticCullParameters, StaticLayouts);

/// What a cut draws, layout by layout.
type StaticLayouts = [StaticDrawParameters; StaticLayout::COUNT];

/// Offers the draw a cascade drawn whole wants this frame, while the trees stand still: where its box moved, the scene
/// changed under it or the sway shows.
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
  let is_changed: bool = cascade.drawn.is_none_or(|(version, contents)| {
    version != fit.version || scene.is_changed_since(contents, |sphere| fit.view.reaches(sphere))
  });

  if !(is_changed || frame.sway.is_redrawn(frame.sway.reach, fit.texel, cascade.drawn_at)) {
    cascade.waited = 0;

    return None;
  }

  Some(offer(index, cascade, frame, scheduler, (CascadeDraw::Whole, fit)))
}

/// Offers the draw a cascade keeping a static layer wants this frame, while the trees sway: over its static layer, at
/// the fit that layer was drawn at, while the cascade's fit holds by it and nothing changed under it (the nearest
/// cascade's only while its fit is the same); else a new static layer at the cascade's fit.
fn offer_split(
  index: usize,
  cascade: &mut ShadowCascadeView,
  frame: &ShadowFrame<'_>,
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

    is_fit && !scene.is_changed_since(contents, |sphere| static_fit.view.reaches(sphere))
  });
  let is_wanted: bool = cascade.drawn.is_none()
    || !is_held
    || cascade.is_swapped
    || frame.sway.is_redrawn(frame.sway.reach, current.texel, cascade.drawn_at);

  if !is_wanted {
    cascade.waited = 0;

    return None;
  }

  let draw: (CascadeDraw, SunCascade) = if is_held {
    (CascadeDraw::Swaying, static_fit)
  } else {
    (CascadeDraw::Static, current)
  };

  Some(offer(index, cascade, frame, scheduler, draw))
}

/// Offers a cascade's draw `draw` at a fit: at once for the nearest, one never drawn, staggering off, or one whose
/// box strayed past its map; else within its stagger.
fn offer(
  index: usize,
  cascade: &ShadowCascadeView,
  frame: &ShadowFrame<'_>,
  scheduler: &mut PeriodicScheduler,
  (draw, fit): (CascadeDraw, SunCascade),
) -> CascadeOffer {
  let (passes, seed): (&'static [&'static str], f32) = match draw {
    CascadeDraw::Whole => (&WHOLE_PASSES[index], WHOLE_SEEDS[index]),
    CascadeDraw::Static => (&STATIC_PASSES[index], WHOLE_SEEDS[index]),
    CascadeDraw::Swaying => (&SWAY_PASSES[index], SWAY_SEEDS[index]),
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

  CascadeOffer {
    index,
    ticket: scheduler.offer(job),
    draw,
    fit,
    contents: frame.scene.get_contents(),
  }
}

/// Shows a cascade's static layer drawn ahead in place of the one shown, once every band of it is drawn.
fn swap_prefetched(index: usize, cascade: &mut ShadowCascadeView) {
  let Some(prefetch) = cascade.prefetch.filter(|prefetch| prefetch.bands == BANDS) else {
    return;
  };

  cascade.static_layer = get_hidden_layer(index, cascade.static_layer);
  cascade.static_fit = prefetch.fit;
  cascade.drawn_static = Some((prefetch.fit.version, prefetch.contents));
  cascade.prefetch = None;
  cascade.is_swapped = true;
}

/// Offers a band of a far cascade's next static layer, drawn ahead once its fit strayed past `PREFETCH_HOLD` of what its
/// static layer holds, at the fit it strayed to; started again where the fit strays as far from that or the scene
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

  if !current.is_held_within(&prefetch.fit, PREFETCH_HOLD)
    || scene.is_changed_since(prefetch.contents, |sphere| prefetch.fit.view.reaches(sphere))
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

impl ViewFeature for LevelShadows {
  /// The maps and the static layers, while held.
  fn get_memory(&self) -> u64 {
    [&self.maps, &self.statics]
      .into_iter()
      .flatten()
      .map(|maps| u64::from(maps.resolution).pow(2) * maps.layers.len() as u64 * 4)
      .sum()
  }

  fn release(&mut self) {
    self.maps = None;
    self.statics = None;

    for (index, cascade) in self.cascades.iter_mut().enumerate() {
      cascade.drawn = None;
      cascade.forget_static(index);
    }
  }
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
        to_layer(from.texture, source),
        to_layer(into.texture, destination),
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
