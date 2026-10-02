use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use glam::{IVec2, IVec4, Vec3};
use xrf_error::XrfResult;

use crate::camera::camera_view::CameraView;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_details::RenderLevelDetails;
use crate::host::render_level_source::RenderLevelSource;
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
use crate::scene::texture::texture_cache::TextureCache;

/// Metres a detail slot spans, `dm_slot_size`.
const SLOT_METERS: f32 = 2.0;

/// A level's grass (`CDetailManager`): its slots and models read once on a loader thread, then planted on the GPU every
/// frame in a ring around the camera and drawn into the G-buffer. The ring and lists are built again when the settings
/// outgrow them, or need less than half; a density changed plants every slot again.
pub struct LevelGrass {
  pending: Option<Receiver<XrfResult<Option<RenderLevelDetails>>>>,
  level: Option<GrassLevel>,
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

impl LevelGrass {
  pub fn new(device: &wgpu::Device, source: &Arc<dyn RenderLevelSource>) -> Self {
    let (sender, receiver) = channel();
    let source: Arc<dyn RenderLevelSource> = Arc::clone(source);

    rayon::spawn(move || {
      let read: XrfResult<Option<RenderLevelDetails>> = source.read_details();

      if let Err(error) = &read {
        log::warn!("The level's grass cannot be drawn: {error}");
      }

      let _ = sender.send(read);
    });

    let uniform = |label: &str, size: usize| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    Self {
      pending: Some(receiver),
      level: None,
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

  /// Takes the level's grass once it is read, asking for its textures; answers their slots the first time.
  pub fn poll(
    &mut self,
    device: &wgpu::Device,
    pass: &GrassPass,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> Option<Vec<u32>> {
    let read: XrfResult<Option<RenderLevelDetails>> = self.pending.as_ref()?.try_recv().ok()?;

    self.pending = None;

    let details: RenderLevelDetails = read.ok().flatten()?;
    let level: GrassLevel = GrassLevel::new(device, pass, &self.uniform, &details, (textures, source));
    let slots: Vec<u32> = level.texture_slots.clone();

    log::info!(
      "Native viewport grass {} models over {}x{} slots",
      level.model_count,
      level.grid[0],
      level.grid[1]
    );
    self.level = Some(level);

    Some(slots)
  }

  /// Fits the planting to the settings and the camera, building the ring and lists again where the settings outgrow
  /// them; `discard_below` is `r_ssaDISCARD` as the static cull compares it, and `time` the seconds the wind blows by.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &GrassPass,
    settings: &RenderGrassSettings,
    camera: &CameraView,
    discard_below: f32,
    (time, is_windy): (f32, bool),
  ) {
    let wind: GrassWindUniform = self.wind.advance(time, is_windy);

    self.is_drawn = false;

    let Some(level) = self
      .level
      .as_ref()
      .filter(|level| settings.is_enabled && level.model_count > 0)
    else {
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
      let build: GrassBuild = GrassBuild::new(device, pass, wanted);

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

  /// Plants the frame's grass, where it is drawn.
  pub fn plant(&self, encoder: &mut wgpu::CommandEncoder, pass: &GrassPass) {
    let (Some(level), Some(build)) = (&self.level, &self.build) else {
      return;
    };

    if !self.is_drawn {
      return;
    }

    let line: u32 = self.values.reach as u32 * 2 + 1;

    pass.plant(
      encoder,
      (&level.bind_group, &build.bind_group),
      GrassDispatch {
        cells: line * line,
        bands: build.size.bands,
        models: level.model_count,
        capacity: build.size.capacity,
      },
    );
  }

  /// Draws what the frame planted into the G-buffer.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    pass: &GrassPass,
    (targets, view): (&ViewTargets, &ViewBinding),
    textures: &wgpu::BindGroup,
  ) {
    let (Some(level), Some(draw_group)) = (&self.level, &self.draw_group) else {
      return;
    };

    if !self.is_drawn {
      return;
    }

    pass.draw(
      encoder,
      targets,
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
  }
}
