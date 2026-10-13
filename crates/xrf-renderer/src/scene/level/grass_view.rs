use glam::{IVec2, IVec4, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_applied_grass::RenderAppliedGrass;
use crate::contract::render_grass_report::RenderGrassReport;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::grass_card_handles::GrassCardHandles;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::lighting::grass_wind::GrassWind;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::grass_dispatch::GrassDispatch;
use crate::pass::grass_draws::GrassDraws;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::grass_pass::GrassPass;
use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;
use crate::pass::grass_uniform::GrassUniform;
use crate::pass::grass_wind_uniform::GrassWindUniform;
use crate::pass::trace_grass_uniform::TraceGrassUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::grass_build::GrassBuild;
use crate::scene::level::grass_build_size::GrassBuildSize;
use crate::scene::level::grass_cards::GrassCards;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;

/// Metres a detail slot spans, `dm_slot_size`.
const SLOT_METERS: f32 = 2.0;

/// A level's grass as one view plants it: every frame in a ring around its camera, the ring caching what it planted, and
/// drawn into the G-buffer swaying in the view's wind. The ring and lists are built again when the settings outgrow
/// them, or need less than half; a density changed plants every slot again.
pub struct GrassView {
  build: Option<GrassBuild>,
  uniform: wgpu::Buffer,
  values: GrassUniform,
  wind: GrassWind,
  wind_buffer: wgpu::Buffer,
  /// Where the puddles stand, which no tuft is planted in.
  puddles_buffer: wgpu::Buffer,
  draw_group: Option<wgpu::BindGroup>,
  /// The steps, jitter and slope limit the planting last ran with, which a change of plants every slot again.
  planted: Option<(i32, u32, u32)>,
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
    let uniform = |label: &str, size: usize| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    Self {
      build: None,
      uniform: uniform("grass", size_of::<GrassUniform>()),
      values: GrassUniform {
        generation: 0,
        ..Default::default()
      },
      wind: GrassWind::default(),
      wind_buffer: uniform("grass wind", size_of::<GrassWindUniform>()),
      puddles_buffer: uniform("grass puddles", size_of::<GrassPuddlesUniform>()),
      draw_group: None,
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

  /// Fits the planting to the settings and the camera, building the ring and lists again where the settings outgrow
  /// them, and the cards where the grass casts shadows, traced where the view traces its light (`is_traced`) and drawn
  /// into the sun's cascades where they shadow the frame (`cascade_reach`, as far as they reach); `discard_below` is
  /// `r_ssaDISCARD` as the static cull compares it, `time` the seconds the wind blows by, and `strength` the wind's,
  /// `wind_strength_factor`.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    (device, encoder): (&wgpu::Device, &mut FrameEncoder),
    (pass, level): (&GrassPass, Option<&GrassLevel>),
    settings: &RenderGrassSettings,
    camera: &CameraView,
    discard_below: f32,
    (time, is_windy, strength): (f32, bool, f32),
    foliage: &FoliageWindValues,
    puddles: (&wgpu::TextureView, GrassPuddlesUniform),
    (is_traced, cascade_reach): (bool, Option<f32>),
  ) {
    let wind: GrassWindUniform = self
      .wind
      .advance(time, is_windy, strength)
      .with_foliage(foliage)
      .with_alignment(settings.is_aligned);

    self.is_drawn = false;
    self.trace = TraceGrassUniform::default();

    let Some(level) = level.filter(|level| settings.is_enabled && level.model_count > 0) else {
      self.cards = None;

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
      self.cards = Some(GrassCards::new(device, pass, (level, &self.wind_buffer), capacity));
    }

    // `dm_current_size`, `dm_current_fade` and `d_size`, from `r__detail_radius` and `r__detail_density`.
    let density: f32 = settings.density.clamp(0.1, 0.99);
    let steps: i32 = (SLOT_METERS / density).ceil() as i32;
    let jitter: f32 = density / 1.7;
    let reach: i32 = (settings.get_radius() / 4.0).floor() as i32 * 2;
    let wanted: GrassBuildSize = GrassBuildSize::wanted(
      reach as u32,
      ((steps + 1) * (steps + 1)) as u32,
      device.limits().max_storage_buffer_binding_size,
    );

    if self.build.as_ref().is_none_or(|build| !build.size.is_fitting(&wanted)) {
      let build: GrassBuild = GrassBuild::new(device, pass, wanted, &self.uniform, (puddles.0, &self.puddles_buffer));

      self.draw_group = Some(pass.create_draw_group(device, build.get_sorted(), &level.models, &self.wind_buffer));
      self.build = Some(build);
      // A ring made anew holds nothing, so every slot is planted into it.
      self.planted = None;
    }

    if let Some(build) = self.build.as_mut() {
      build.bind_puddles(device, pass, &self.uniform, (puddles.0, &self.puddles_buffer));
    }

    // A slope's limit decides which tufts stand, so a change of it plants every slot again too.
    let slope_limit: f32 = settings.slope_limit.clamp(0.0, 90.0).to_radians();

    if self.planted != Some((steps, jitter.to_bits(), slope_limit.to_bits())) {
      self.planted = Some((steps, jitter.to_bits(), slope_limit.to_bits()));
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
      eye: eye.extend(1.0),
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
    encoder.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.values));
    encoder.write_buffer(&self.wind_buffer, 0, bytemuck::bytes_of(&wind));
    encoder.write_buffer(&self.puddles_buffer, 0, bytemuck::bytes_of(&puddles.1));
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
      .map(|cards| &cards.blas)
  }

  /// Metres from the camera the grass casts the sun's shadow to through its cascades this frame, none where it does
  /// not.
  pub fn get_cascade_reach(&self) -> Option<f32> {
    (self.cards.is_some() && self.is_drawn && self.is_card_cascaded).then_some(self.values.card_reach)
  }

  /// The cards as the cascades draw them, stood this frame (`cards`), where they do.
  pub fn get_shadow_casters<'a>(
    &'a self,
    (pass, textures): (&'a GrassPass, &'a wgpu::BindGroup),
    cards: Option<GrassCardHandles>,
  ) -> Option<GrassShadowCasters<'a>> {
    let group: &'a wgpu::BindGroup = &self.cards.as_ref().filter(|_| self.is_card_cascaded)?.shadow_group;

    Some(GrassShadowCasters {
      pass,
      group,
      textures,
      cards: cards?,
    })
  }

  /// Stands the cards of `level` about the eye, sizes the cascades' draw of them, and builds their structure where the
  /// view traces them; none where the grass casts no shadow.
  pub fn add_cards<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
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
    let tufts: u32 = GrassCards::get_window_cells(self.values.card_slots as u32) * build.size.per_cell;

    graph
      .add_compute_pass("grass cards")
      .buffer(vertices, GraphBufferAccess::StorageReadWrite)
      .buffer(triangles, GraphBufferAccess::StorageReadWrite)
      .buffer(draw, GraphBufferAccess::StorageReadWrite)
      .record(move |context| {
        pass.record_cards(
          context.get_pass(),
          (&level.bind_group, &build.bind_group, &cards.group),
          // Only the structure built over them reads the unused triangles, marked as none; the cascades draw the count.
          (if self.is_card_traced { cards.capacity } else { 0 }, tufts),
        );
      });

    let count: GraphBuffer = bindings.import_buffer(graph, "grass card count", &cards.count);

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

    if !self.is_card_traced {
      return Some(handles);
    }

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
          blas: &cards.blas,
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

  /// Plants the frame's grass of `level` where it is drawn: the ring's stale cells, ahead of the frame's depth.
  pub fn add_planting<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
  ) -> Option<GraphBuffer> {
    let (Some(level), Some(build)) = (level, &self.build) else {
      return None;
    };

    if !self.is_drawn {
      return None;
    }

    let dispatch: GrassDispatch = self.get_dispatch(level, build);
    let args: GraphBuffer = bindings.import_buffer(graph, "grass draw arguments", &level.args);

    graph
      .add_compute_pass("grass planting")
      .buffer(args, GraphBufferAccess::StorageReadWrite)
      .record(move |context| {
        pass.record_plant(context.get_pass(), (&level.bind_group, &build.bind_group), dispatch);
      });

    Some(args)
  }

  /// Culls what the ring holds by the view and the frame's depth `occlusion`, into the draw arguments `args` the
  /// planting answered, then copies its total out for reading back: an effect the graph cannot see.
  pub fn add_cull<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
    (args, occlusion): (GraphBuffer, GrassOcclusionParameters),
  ) {
    let (Some(level), Some(build)) = (level, &self.build) else {
      return;
    };
    let dispatch: GrassDispatch = self.get_dispatch(level, build);
    let counts: GraphBuffer = bindings.import_buffer(graph, "grass counts", &level.counts);

    graph
      .add_compute_pass("grass cull")
      .buffer(args, GraphBufferAccess::StorageReadWrite)
      .buffer(counts, GraphBufferAccess::StorageReadWrite)
      .parameters(&occlusion)
      .record(move |context| {
        pass.record_cull(
          context,
          (
            &level.bind_group,
            &build.bind_group,
            (&build.sort_group, &build.sort_dispatch),
          ),
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

  /// Declares the draw of what the frame planted, its arguments `args`, into the G-buffer.
  pub fn add_draw<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (pass, level): (&'a GrassPass, Option<&'a GrassLevel>),
    (targets, args): (ViewTargetHandles, GraphBuffer),
    (view, textures): (&'a ViewBinding, &'a wgpu::BindGroup),
  ) {
    let (Some(level), Some(draw_group)) = (level, &self.draw_group) else {
      return;
    };

    targets
      .get_gbuffer()
      .into_iter()
      .fold(graph.add_raster_pass("grass"), |builder, texture| {
        builder.color(GraphColorAttachment::new(texture, wgpu::LoadOp::Load))
      })
      .depth(GraphDepthAttachment::new(targets.depth, wgpu::LoadOp::Load))
      .buffer(args, GraphBufferAccess::Indirect)
      .record(move |context| {
        pass.record_draw(
          context.get_pass(),
          view,
          (draw_group, textures),
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
  /// Its uniforms, and the ring, lists and cards while it plants.
  fn get_memory(&self) -> u64 {
    let uniforms: u64 = [&self.uniform, &self.wind_buffer, &self.puddles_buffer]
      .into_iter()
      .map(wgpu::Buffer::size)
      .sum();

    uniforms
      + self.build.as_ref().map_or(0, GrassBuild::get_memory)
      + self.cards.as_ref().map_or(0, GrassCards::get_memory)
  }

  fn release(&mut self) {
    self.build = None;
    self.draw_group = None;
    self.planted = None;
    self.cards = None;
  }
}
