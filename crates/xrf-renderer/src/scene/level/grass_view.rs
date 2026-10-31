use glam::{IVec2, IVec4, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDeclaration, GraphDepthAttachment,
  GraphTexture, StorageArray, StorageArrayMut, StorageValue, StorageValueMut, UniformBinding,
};

use crate::contract::render_applied_grass::RenderAppliedGrass;
use crate::contract::render_grass_report::RenderGrassReport;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::lighting::grass_wind::GrassWind;
use crate::pass::grass_build_parameters::GrassBuildParameters;
use crate::pass::grass_card_shadow_parameters::GrassCardShadowParameters;
use crate::pass::grass_cards_parameters::GrassCardsParameters;
use crate::pass::grass_dispatch::GrassDispatch;
use crate::pass::grass_draw_parameters::GrassDrawParameters;
use crate::pass::grass_draws::GrassDraws;
use crate::pass::grass_level_parameters::GrassLevelParameters;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::grass_pass::GrassPass;
use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;
use crate::pass::grass_sort_parameters::GrassSortParameters;
use crate::pass::grass_uniform::GrassUniform;
use crate::pass::grass_wind_uniform::GrassWindUniform;
use crate::pass::trace_grass_uniform::TraceGrassUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::grass_build::GrassBuild;
use crate::scene::level::grass_build_size::GrassBuildSize;
use crate::scene::level::grass_card_handles::GrassCardHandles;
use crate::scene::level::grass_cards::GrassCards;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::grass_planting::GrassPlanting;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::grass_view_frame::GrassViewFrame;

/// Metres a detail slot spans, `dm_slot_size`.
const SLOT_METERS: f32 = 2.0;

/// A level's grass as one view plants it: every frame in a ring around its camera, the ring caching what it planted,
/// and drawn into the G-buffer swaying in the view's wind. The ring and lists are built again when the settings outgrow
/// them, or need less than half; a density changed plants every slot again.
pub struct GrassView {
  build: Option<GrassBuild>,
  values: GrassUniform,
  wind: GrassWind,
  /// How the grass sways this frame.
  sway: GrassWindUniform,
  /// This frame's puddle sites, and where they stand, which no tuft is planted in.
  puddles: Option<(wgpu::TextureView, GrassPuddlesUniform)>,
  /// The grass level, steps, jitter and slope limit the planting last ran with, a change of any of which plants every
  /// slot again.
  planted: Option<(u64, i32, u32, u32)>,
  /// Whether this frame plants and draws.
  is_drawn: bool,
  /// The cards casting the grass's shadows, while it casts them, and whether the view traces them and whether its sun
  /// cascades draw them this frame.
  cards: Option<GrassCards>,
  is_card_traced: bool,
  is_card_cascaded: bool,
  /// The planting as a traced hit reads it, no models while none is drawn.
  trace: TraceGrassUniform,
  /// The cull's total, read back without waiting on the GPU.
  stats: StatsReadback,
  /// How many triangles the cards' tufts asked for, read back a frame or more late, which their room is sized to.
  card_demand: StatsReadback,
}

impl GrassView {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      build: None,
      values: GrassUniform {
        generation: 0,
        ..Default::default()
      },
      wind: GrassWind::default(),
      sway: GrassWindUniform::default(),
      puddles: None,
      planted: None,
      is_drawn: false,
      cards: None,
      is_card_traced: false,
      is_card_cascaded: false,
      trace: TraceGrassUniform::default(),
      stats: StatsReadback::new(device),
      card_demand: StatsReadback::new(device),
    }
  }

  /// Starts the engine's sway again from nothing, as at a view's first frame.
  pub fn restart_wind(&mut self) {
    self.wind = GrassWind::default();
  }

  /// Fits the planting to the settings and the camera, building the ring and lists again where the settings outgrow
  /// them, and the cards where the grass casts shadows, traced where the view traces its light and drawn into the sun's
  /// cascades where they shadow the frame.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    level: Option<&GrassLevel>,
    settings: &RenderGrassSettings,
    frame: &GrassViewFrame<'_>,
  ) {
    let GrassViewFrame {
      camera,
      discard_below,
      time,
      is_windy,
      strength,
      foliage,
      puddles,
      is_traced,
      cascade_reach,
    } = *frame;
    let wind: GrassWindUniform = self
      .wind
      .advance(time, is_windy, strength)
      .with_foliage(foliage)
      .with_alignment(settings.is_aligned);

    self.is_drawn = false;
    self.trace = TraceGrassUniform::default();

    // Off, it holds nothing: its ring and lists are made again, and every slot planted, once it is back.
    let Some(level) = level.filter(|level| settings.is_enabled && level.model_count > 0) else {
      self.release();

      return;
    };

    let is_cascaded: bool = cascade_reach.is_some();

    self.is_card_traced = is_traced;
    self.is_card_cascaded = is_cascaded;

    // Made again where the radius or what their tufts ask wants them to hold another number of triangles.
    let capacity: u32 = GrassCards::get_capacity(
      self.cards.as_ref().map_or(0, |cards| cards.capacity),
      self.card_demand.take()[0],
    );

    if !(is_traced || is_cascaded) || !settings.is_shadowed {
      self.cards = None;
    } else if self.cards.as_ref().is_none_or(|cards| cards.capacity != capacity) {
      self.cards = Some(GrassCards::new(device, capacity));
    }

    // `dm_current_size`, `dm_current_fade` and `d_size`, from `r__detail_radius` and `r__detail_density`.
    let reach: i32 = (settings.get_radius() / 4.0).floor() as i32 * 2;
    let density: f32 = Self::get_density(
      settings.density.clamp(0.1, 0.99),
      (reach, camera.is_orthographic()),
      device.limits().max_storage_buffer_binding_size,
    );
    let steps: i32 = (SLOT_METERS / density).ceil() as i32;
    let jitter: f32 = density / 1.7;
    let wanted: GrassBuildSize = GrassBuildSize::wanted(
      reach as u32,
      ((steps + 1) * (steps + 1)) as u32,
      device.limits().max_storage_buffer_binding_size,
    );

    if self.build.as_ref().is_none_or(|build| !build.size.is_fitting(&wanted)) {
      self.build = Some(GrassBuild::new(device, wanted));
      // A ring made anew holds nothing, so every slot is planted into it.
      self.planted = None;
    }

    // A slope's limit decides which tufts stand, and the scene's grass replaced holds other slots and models, so a
    // change of either plants every slot again too.
    let slope_limit: f32 = settings.slope_limit.clamp(0.0, 90.0).to_radians();
    let planted: (u64, i32, u32, u32) = (level.key, steps, jitter.to_bits(), slope_limit.to_bits());

    if self.planted != Some(planted) {
      self.planted = Some(planted);
      self.values.generation += 1;
    }

    let Some(build) = &self.build else {
      return;
    };
    let eye: Vec3 = Vec3::new(camera.position.x, camera.position.y, -camera.position.z);
    // Traced, the cards shadow as far as their radius; drawn into the cascades alone, as far as those reach.
    let (card_reach, card_fade): (f32, f32) =
      GrassCards::get_reach(settings, if is_traced { None } else { cascade_reach });

    self.values = GrassUniform {
      eye: eye.extend(if camera.is_orthographic() { 0.0 } else { 1.0 }),
      planes: camera.get_planes(),
      center: IVec2::new(
        (eye.x / SLOT_METERS + 0.5).floor() as i32,
        (eye.z / SLOT_METERS + 0.5).floor() as i32,
      ),
      reach,
      steps,
      grid: IVec4::from_array(level.grid),
      fade: 2.0 * reach as f32 - 0.5,
      jitter,
      height: settings.height,
      discard_below,
      per_cell: build.size.per_cell,
      bands: build.size.bands,
      capacity: build.size.capacity,
      model_count: level.model_count,
      tuft_reach: level.tuft_reach,
      slope_limit,
      card_reach,
      card_fade,
      card_slots: GrassCards::get_slots(card_reach) as i32,
      ..self.values
    };
    self.sway = wind;
    self.puddles = Some((puddles.0.clone(), puddles.1));
    self.is_drawn = true;
    self.trace = TraceGrassUniform {
      grid: self.values.grid,
      steps,
      fade: self.values.fade,
      height: settings.height,
      model_count: level.model_count,
      slope_limit,
      _pad: [0.0; 3],
    };
  }

  /// The cards' structure where the view traces the grass's shadows this frame.
  pub fn get_cards_blas(&self) -> Option<&wgpu::Blas> {
    self
      .cards
      .as_ref()
      .filter(|_| self.is_drawn && self.is_card_traced)
      .and_then(|cards| cards.blas.as_ref())
  }

  /// The spacing the tufts are planted at: `density`'s, or, under a lens of parallel rays, as much wider as the lists
  /// need to hold every slot's within `reach`. A view through a lens keeps its spacing and loses its farthest bands
  /// where the lists run out, its least seen; a level map's tile must plant all of its ground alike.
  pub(crate) fn get_density(density: f32, (reach, is_flat): (i32, bool), binding_limit: u64) -> f32 {
    if !is_flat {
      return density;
    }

    let line: u64 = u64::from(reach.unsigned_abs()) * 2 + 1;
    let room: u64 = binding_limit / GrassBuildSize::ITEM_BYTES / line.saturating_mul(line).max(1);
    // A slot's candidates stand `steps + 1` a side, so the most that fit stand the root of the room less one apart.
    let steps: f32 = ((room as f32).sqrt().floor() - 1.0).max(1.0);

    density.max(SLOT_METERS / steps).min(0.99)
  }

  /// Metres from the camera the grass casts the sun's shadow to through its cascades this frame, none where it does
  /// not.
  pub fn get_cascade_reach(&self) -> Option<f32> {
    (self.cards.is_some() && self.is_drawn && self.is_card_cascaded).then_some(self.values.card_reach)
  }

  /// The cards of `level` as the cascades draw them, stood this frame (`cards`), where they do.
  pub fn get_shadow_casters<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    (pass, level, textures): (&'a GrassPass, Option<&'a GrassLevel>, &'a wgpu::BindGroup),
    cards: Option<GrassCardHandles>,
  ) -> Option<GrassShadowCasters<'a>> {
    self.cards.as_ref().filter(|_| self.is_card_cascaded)?;

    let (level, cards): (&'a GrassLevel, GrassCardHandles) = (level?, cards?);

    Some(GrassShadowCasters {
      pass,
      parameters: GrassCardShadowParameters {
        card_vertices: StorageArray::new(cards.vertices),
        card_triangles: StorageArray::new(cards.triangles),
        models: StorageArray::new(bindings.import_buffer(graph, "grass models", &level.models)),
      },
      textures,
      draw: cards.draw,
    })
  }

  /// Stands the cards of `level` about the eye, sizes the cascades' draw of them, and builds their structure where the
  /// view traces them; none where the grass casts no shadow.
  pub fn add_cards<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
    planting: GrassPlanting,
  ) -> Option<GrassCardHandles> {
    let (Some(level), Some(build), Some(cards)) = (level, &self.build, self.cards.as_ref()) else {
      return None;
    };

    if !self.is_drawn {
      return None;
    }

    let vertices: GraphBuffer = bindings.import_buffer(graph, "grass card vertices", &cards.vertices);
    let triangles: GraphBuffer = bindings.import_buffer(graph, "grass card triangles", &cards.triangles);
    let draw: GraphBuffer = bindings.import_buffer(graph, "grass card draw", &cards.draw);
    let count: GraphBuffer = bindings.import_buffer(graph, "grass card count", &cards.count);
    let parameters: GrassCardsParameters = GrassCardsParameters {
      positions: StorageArray::new(bindings.import_buffer(graph, "grass positions", &level.positions)),
      uvs: StorageArray::new(bindings.import_buffer(graph, "grass coordinates", &level.uvs)),
      indices: StorageArray::new(bindings.import_buffer(graph, "grass indices", &level.indices)),
      wind: planting.draw.wind,
      card_vertices: StorageArrayMut::new(vertices),
      card_triangles: StorageArrayMut::new(triangles),
      card_count: StorageValueMut::new(count),
      card_draw: StorageValueMut::new(draw),
    };
    let tufts: u32 = GrassCards::get_window_cells(self.values.card_slots as u32) * build.size.per_cell;

    graph
      .add_compute_pass("grass cards")
      .parameters(&planting.level)
      .parameters(&planting.build)
      .parameters(&parameters)
      .record(move |context| {
        pass.record_cards(
          context,
          (&planting.level, &planting.build, &parameters),
          // Only the structure built over them reads the unused triangles, marked as none; the cascades draw the count.
          (if self.is_card_traced { cards.capacity } else { 0 }, tufts),
        );
      });

    // How many triangles the tufts asked for, past the room too, read back for the cards' room.
    graph
      .add_encoder_pass("grass card demand")
      .buffer(draw, GraphBufferAccess::StorageRead)
      .buffer(count, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        let count: &wgpu::Buffer = context.get_buffer(count);

        self.card_demand.record(context.get_encoder(), count, 0);
      });

    let handles: GrassCardHandles = GrassCardHandles {
      vertices,
      triangles,
      draw,
    };

    let Some(blas) = cards.blas.as_ref().filter(|_| self.is_card_traced) else {
      return Some(handles);
    };

    graph
      .add_encoder_pass("grass cards build")
      .buffer(vertices, GraphBufferAccess::StorageRead)
      .keep()
      .record(move |context| {
        let size: wgpu::BlasTriangleGeometrySizeDescriptor = GrassCards::get_size(cards.capacity);
        let geometry: wgpu::BlasTriangleGeometry<'_> = wgpu::BlasTriangleGeometry {
          size: &size,
          vertex_buffer: context.get_buffer(vertices),
          first_vertex: 0,
          vertex_stride: 12,
          index_buffer: None,
          first_index: None,
          transform_buffer: None,
          transform_buffer_offset: None,
        };
        let entry: wgpu::BlasBuildEntry<'_> = wgpu::BlasBuildEntry {
          blas,
          geometry: wgpu::BlasGeometries::TriangleGeometries(vec![geometry]),
        };

        context
          .get_encoder()
          .build_acceleration_structures(std::iter::once(&entry), std::iter::empty());
      });

    Some(handles)
  }

  /// The planting as a traced hit reads it this frame.
  pub fn get_trace_uniform(&self) -> &TraceGrassUniform {
    &self.trace
  }

  /// What the ring's slots are planted under, moved each time every slot is planted again.
  #[cfg(test)]
  pub fn get_generation(&self) -> u32 {
    self.values.generation
  }

  /// The grass as planted, or none where none is.
  pub fn get_applied(&self, settings: &RenderGrassSettings) -> Option<RenderAppliedGrass> {
    let build = self.build.as_ref().filter(|_| self.is_drawn)?;
    let line: u32 = self.values.reach as u32 * 2 + 1;
    let cells: u32 = line * line;
    let candidates: u32 = ((self.values.steps + 1) * (self.values.steps + 1)) as u32;

    Some(RenderAppliedGrass {
      radius: self.values.reach as f32 * SLOT_METERS,
      density: settings.density.clamp(0.1, 0.99),
      height: settings.height,
      tufts: build.size.capacity.min(cells.saturating_mul(build.size.per_cell)),
      wanted: cells.saturating_mul(candidates),
    })
  }

  /// Asks for the cull's total and the cards' demand copied out with the frame just submitted.
  pub fn request_stats(&self) {
    self.stats.request();
    self.card_demand.request();
  }

  /// Drops the copies made with a frame that was never submitted.
  pub fn abandon_stats(&self) {
    self.stats.abandon();
    self.card_demand.abandon();
  }

  /// What a recent frame drew: the cull's latest total read back, and the most the lists hold.
  pub fn take_report(&mut self) -> RenderGrassReport {
    let [drawn, ..] = self.stats.take();

    RenderGrassReport {
      drawn,
      capacity: self
        .build
        .as_ref()
        .filter(|_| self.is_drawn)
        .map_or(0, |build| build.size.capacity),
    }
  }

  /// Plants the frame's grass of `level` where it is drawn: the ring's stale cells, ahead of the frame's depth. Answers
  /// what the frame's cull, cards and draw bind of it.
  pub fn add_planting<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
  ) -> Option<GrassPlanting> {
    let (Some(level), Some(build), Some((puddles, puddle_window))) = (level, &self.build, &self.puddles) else {
      return None;
    };

    if !self.is_drawn {
      return None;
    }

    let puddle_sites: GraphTexture = bindings.import_view(graph, "grass puddles", puddles);
    let mut import = |label: &'static str, buffer: &'a wgpu::Buffer| bindings.import_buffer(graph, label, buffer);
    let models: GraphBuffer = import("grass models", &level.models);
    let counts: GraphBuffer = import("grass counts", &level.counts);
    let args: GraphBuffer = import("grass draw arguments", &level.args);
    let sorted: GraphBuffer = import("grass sorted", &build.sorted);
    let wind: UniformBinding<GrassWindUniform> = runtime.push_uniform(&self.sway);
    let planting: GrassPlanting = GrassPlanting {
      level: GrassLevelParameters {
        grid: StorageArray::new(import("grass grid", &level.cells)),
        slot_records: StorageArray::new(import("grass slots", &level.slots)),
        bins: StorageArray::new(import("grass bins", &level.bins)),
        triangles: StorageArray::new(import("grass triangles", &level.triangles)),
        dither: StorageValue::new(import("grass dither", &level.dither)),
        models: StorageArray::new(models),
        counts: StorageArrayMut::new(counts),
        cursors: StorageArrayMut::new(import("grass cursors", &level.cursors)),
        args: StorageArrayMut::new(args),
      },
      build: GrassBuildParameters {
        keys: StorageArrayMut::new(import("grass keys", &build.keys)),
        shapes: StorageArrayMut::new(import("grass cell shapes", &build.shapes)),
        cached: StorageArrayMut::new(import("grass cached tufts", &build.cached)),
        band_counts: StorageArrayMut::new(import("grass band counts", &build.band_counts)),
        schedule: StorageValueMut::new(import("grass schedule", &build.schedule)),
        planted: StorageArrayMut::new(import("grass planted", &build.planted)),
        planted_models: StorageArrayMut::new(import("grass planted models", &build.planted_models)),
        sorted: StorageArrayMut::new(sorted),
        params: runtime.push_uniform(&self.values),
        puddles: puddle_sites,
        puddle_window: runtime.push_uniform(puddle_window),
      },
      sort: GrassSortParameters {
        sort_dispatch: StorageValueMut::new(import("grass sort dispatch", &build.sort_dispatch)),
      },
      draw: GrassDrawParameters {
        sorted: StorageArray::new(sorted),
        models: StorageArray::new(models),
        wind,
      },
      args,
      counts,
    };
    let dispatch: GrassDispatch = self.get_dispatch(level, build);

    graph
      .add_compute_pass("grass planting")
      .parameters(&planting.level)
      .parameters(&planting.build)
      .record(move |context| {
        pass.record_plant(context, (&planting.level, &planting.build), dispatch);
      });

    Some(planting)
  }

  /// Culls what the ring holds by the view and the frame's depth `occlusion`, into the draw arguments the planting
  /// answered, then copies its total out for reading back: an effect the graph cannot see.
  pub fn add_cull<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
    (planting, occlusion): (GrassPlanting, GrassOcclusionParameters),
  ) {
    let (Some(level), Some(build)) = (level, &self.build) else {
      return;
    };
    let dispatch: GrassDispatch = self.get_dispatch(level, build);
    let counts: GraphBuffer = planting.counts;

    graph
      .add_compute_pass("grass cull")
      .parameters(&planting.level)
      .parameters(&planting.build)
      .parameters(&occlusion)
      .parameters(&planting.sort)
      .record(move |context| {
        pass.record_cull(
          context,
          (&planting.level, &planting.build, &planting.sort),
          &occlusion,
          dispatch,
        );
      });
    graph
      .add_encoder_pass("grass stats")
      .buffer(counts, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        let counts: &wgpu::Buffer = context.get_buffer(counts);

        self
          .stats
          .record(context.get_encoder(), counts, u64::from(level.model_count) * 4);
      });
  }

  /// What the planting and the cull dispatch over: the ring's cells, its bands, the models and the items.
  fn get_dispatch(&self, level: &GrassLevel, build: &GrassBuild) -> GrassDispatch {
    let line: u32 = self.values.reach as u32 * 2 + 1;

    GrassDispatch {
      cells: line * line,
      bands: build.size.bands,
      models: level.model_count,
      capacity: build.size.capacity,
    }
  }

  /// Declares the draw of what the frame planted (`planting`) into the G-buffer.
  pub fn add_draw<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
    (targets, planting): (ViewTargetHandles, GrassPlanting),
    (view, textures): (&'a ViewBinding, &'a wgpu::BindGroup),
  ) {
    let Some(level) = level else {
      return;
    };

    targets
      .get_gbuffer()
      .into_iter()
      .fold(graph.add_raster_pass("grass"), |builder, texture| {
        builder.color(GraphColorAttachment::new(texture, wgpu::LoadOp::Load))
      })
      .depth(GraphDepthAttachment::new(targets.depth, wgpu::LoadOp::Load))
      .buffer(planting.args, GraphBufferAccess::Indirect)
      .parameters(&planting.draw)
      .record(move |context| {
        pass.record_draw(
          context,
          view,
          (&planting.draw, textures),
          &GrassDraws {
            positions: &level.positions,
            uvs: &level.uvs,
            indices: &level.indices,
            args: &level.args,
            models: level.model_count,
          },
        );
      });
  }
}

impl ViewFeature for GrassView {
  /// The ring, lists and cards while it plants.
  fn get_memory(&self) -> u64 {
    self.build.as_ref().map_or(0, GrassBuild::get_memory) + self.cards.as_ref().map_or(0, GrassCards::get_memory)
  }

  fn release(&mut self) {
    self.build = None;
    self.puddles = None;
    self.planted = None;
    self.cards = None;
  }
}
