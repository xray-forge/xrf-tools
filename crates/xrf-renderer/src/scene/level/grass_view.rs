use glam::{IVec2, IVec4, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_applied_grass::RenderAppliedGrass;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::lighting::grass_wind::GrassWind;
use crate::pass::grass_dispatch::GrassDispatch;
use crate::pass::grass_draws::GrassDraws;
use crate::pass::grass_pass::GrassPass;
use crate::pass::grass_uniform::GrassUniform;
use crate::pass::grass_wind_uniform::GrassWindUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::grass_build::GrassBuild;
use crate::scene::level::grass_build_size::GrassBuildSize;
use crate::scene::level::grass_level::GrassLevel;

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
  draw_group: Option<wgpu::BindGroup>,
  /// The steps and jitter the planting last ran with, which a change of plants every slot again.
  planted: Option<(i32, u32)>,
  /// Whether this frame plants and draws.
  is_drawn: bool,
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
      draw_group: None,
      planted: None,
      is_drawn: false,
    }
  }

  /// Fits the planting to the settings and the camera, building the ring and lists again where the settings outgrow
  /// them; `discard_below` is `r_ssaDISCARD` as the static cull compares it, `time` the seconds the wind blows by,
  /// and `strength` the wind's, `wind_strength_factor`.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (pass, level): (&GrassPass, Option<&GrassLevel>),
    settings: &RenderGrassSettings,
    camera: &CameraView,
    discard_below: f32,
    (time, is_windy, strength): (f32, bool, f32),
  ) {
    let wind: GrassWindUniform = self.wind.advance(time, is_windy, strength);

    self.is_drawn = false;

    let Some(level) = level.filter(|level| settings.is_enabled && level.model_count > 0) else {
      return;
    };
    // `dm_current_size`, `dm_current_fade` and `d_size`, from `r__detail_radius` and `r__detail_density`.
    let density: f32 = settings.density.clamp(0.1, 0.99);
    let steps: i32 = (SLOT_METERS / density).ceil() as i32;
    let jitter: f32 = density / 1.7;
    let reach: i32 = (settings.radius / 4.0).floor().max(0.0) as i32 * 2;
    let wanted: GrassBuildSize = GrassBuildSize::wanted(
      reach as u32,
      ((steps + 1) * (steps + 1)) as u32,
      device.limits().max_storage_buffer_binding_size,
    );

    if self.build.as_ref().is_none_or(|build| !build.size.is_fitting(&wanted)) {
      let build: GrassBuild = GrassBuild::new(device, pass, wanted, &self.uniform);

      self.draw_group = Some(pass.create_draw_group(device, &build.sorted, &level.models, &self.wind_buffer));
      self.build = Some(build);
      // A ring made anew holds nothing, so every slot is planted into it.
      self.planted = None;
    }

    if self.planted != Some((steps, jitter.to_bits())) {
      self.planted = Some((steps, jitter.to_bits()));
      self.values.generation += 1;
    }

    let Some(build) = &self.build else {
      return;
    };
    let eye: Vec3 = Vec3::new(camera.position.x, camera.position.y, -camera.position.z);

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
      ..self.values
    };
    queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.values));
    queue.write_buffer(&self.wind_buffer, 0, bytemuck::bytes_of(&wind));
    self.is_drawn = true;
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

  /// Plants the frame's grass of `level`, where it is drawn.
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

    let line: u32 = self.values.reach as u32 * 2 + 1;
    let dispatch: GrassDispatch = GrassDispatch {
      cells: line * line,
      bands: build.size.bands,
      models: level.model_count,
      capacity: build.size.capacity,
    };
    let args: GraphBuffer = bindings.import_buffer(graph, "grass draw arguments", &level.args);

    graph
      .add_compute_pass("grass planting")
      .buffer(args, GraphBufferAccess::StorageReadWrite)
      .record(move |context| {
        pass.record_plant(context.get_pass(), (&level.bind_group, &build.bind_group), dispatch);
      });

    Some(args)
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
