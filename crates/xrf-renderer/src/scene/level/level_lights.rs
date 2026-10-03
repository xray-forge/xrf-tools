use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

use glam::{Mat4, Vec3, Vec4};
use xrf_math::EPS_L;
use xrf_visual::{LightDescription, LightKind, LightsDescription};

use crate::camera::camera_view::CameraView;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::frame::stats_readback::StatsReadback;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::lighting::light_animation::to_animated_color;
use crate::lighting::light_basis::{LightBasis, to_light_lod};
use crate::lighting::light_shadow_size::{LIGHT_SHADOW_POINT_FACES, to_light_shadow_scale};
use crate::lighting::light_specular::to_light_specular;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_buffers::LightBuffers;
use crate::pass::light_record::{LIGHT_NO_CONE, LIGHT_NO_PROJECTOR, LightRecord};
use crate::pass::lights_uniform::LightsUniform;
use crate::scene::level::level_light_shadows::{LIGHT_SHADOW_ATLAS_SIZE, LevelLightShadows};
use crate::scene::level::light_shadow_set::LightShadowSet;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::texture::texture_cache::{MISSING_SLOT, TextureCache};
use crate::scene::texture::texture_role::TextureRole;

/// Lights standing in view at most in one frame: the nearest are kept.
pub const MAX_LIGHTS: usize = 1024;

/// Clusters the view is cut into, and lights one holds at most, as `shaders/common/light_clusters.wgsl` declares them.
const LIGHT_CLUSTERS: u64 = 16 * 9 * 24;
const LIGHT_CLUSTER_CAPACITY: u64 = 64;

/// The bytes after the clusters' counts the binning counts its overflow in, as a readback copies them.
const OVERFLOW_BYTES: u64 = 16;

/// What a light's falloff reaches zero at, a share of its range: `L_R` (`r3_rendertarget_accum_point.cpp`).
const FALLOFF_RANGE: f32 = 0.95;

/// A level's local lights: read once on a loader thread, then each frame the nearest in view written out in view
/// space, animated as the engine animates them, for the lights pass to bin and accumulate.
pub struct LevelLights {
  pending: Option<Receiver<Result<LightsDescription, String>>>,
  description: Option<LightsDescription>,
  /// Each projector's texture slot, by its index among the description's projectors.
  projectors: Vec<u32>,
  records: Vec<LightRecord>,
  pub record_buffer: wgpu::Buffer,
  pub counts: wgpu::Buffer,
  pub items: wgpu::Buffer,
  pub uniform: wgpu::Buffer,
  count: u32,
  /// What the last frame's lights came to, and the binning's count of the clusters it filled, read back.
  report: RenderLightsReport,
  overflow: StatsReadback,
  shadows: LevelLightShadows,
  started: Instant,
  /// What a zone's range strays by each frame.
  random: u64,
}

impl LevelLights {
  pub fn new(
    device: &wgpu::Device,
    view_layout: &wgpu::BindGroupLayout,
    args_size: u64,
    source: &Arc<dyn RenderLevelSource>,
  ) -> Self {
    let (sender, receiver) = channel();
    let source: Arc<dyn RenderLevelSource> = Arc::clone(source);
    let storage = |label: &str, size: u64| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    rayon::spawn(move || {
      let lights: Result<LightsDescription, String> = source.read_lights().map_err(|error| error.to_string());

      if let Err(error) = &lights {
        log::error!("The level's lights cannot be drawn: {error}");
      }

      let _ = sender.send(lights);
    });

    Self {
      pending: Some(receiver),
      description: None,
      projectors: Vec::new(),
      records: Vec::with_capacity(MAX_LIGHTS),
      record_buffer: storage("light records", (MAX_LIGHTS * size_of::<LightRecord>()) as u64),
      // The clusters' counts, then the binning's two words of overflow.
      counts: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("light cluster counts"),
        size: LIGHT_CLUSTERS * 4 + OVERFLOW_BYTES,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      items: storage("light cluster items", LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY * 4),
      uniform: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lights"),
        size: size_of::<LightsUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      count: 0,
      report: RenderLightsReport::default(),
      overflow: StatsReadback::new(device),
      shadows: LevelLightShadows::new(device, view_layout, args_size),
      started: Instant::now(),
      random: 0x9E37_79B9_7F4A_7C15,
    }
  }

  pub fn get_buffers(&self) -> LightBuffers<'_> {
    LightBuffers {
      records: &self.record_buffer,
      counts: &self.counts,
      items: &self.items,
      uniform: &self.uniform,
    }
  }

  /// The texture slots the projectors sample.
  pub fn get_projectors(&self) -> &[u32] {
    &self.projectors
  }

  pub fn get_count(&self) -> u32 {
    self.count
  }

  /// Takes the lights once their loader read them, asking for every projector they name.
  pub fn poll(&mut self, textures: &mut TextureCache, source: &Arc<dyn RenderAssetSource>) {
    let Some(receiver) = &self.pending else {
      return;
    };
    let Ok(lights) = receiver.try_recv() else {
      return;
    };

    self.pending = None;

    if let Ok(lights) = lights {
      self.projectors = lights
        .projectors
        .iter()
        .map(|reference| textures.request(reference, TextureRole::Projector, source))
        .collect();
      log::info!("Native viewport lights {} local lights", lights.lights.len());
      self.description = Some(lights);
    }
  }

  /// Writes out the lights standing in view this frame, nearest first, animated and faded as the engine would, in the
  /// camera's view space; a shadowed one only once its faces are drawn, with their squares of the atlas.
  ///
  /// `lod` is the progressive meshes' `start` and `end` screen areas, which a shadowed light fades between; `contents`
  /// counts what the scene holds, which a face drawn with less is drawn again for; `sway` has a face over swaying trees
  /// drawn again.
  pub fn prepare(
    &mut self,
    queue: &wgpu::Queue,
    camera: &CameraView,
    settings: &RenderLightsSettings,
    lod: (f32, f32),
    contents: usize,
    sway: &ShadowSway<'_>,
  ) {
    self.records.clear();
    self.shadows.begin();
    self.report = RenderLightsReport::default();

    let is_shadowing: bool = settings.is_enabled && settings.is_shadowed;

    if settings.is_enabled
      && let Some(description) = &self.description
    {
      let planes: [Vec4; 6] = camera.get_planes();
      let seconds: f32 = self.started.elapsed().as_secs_f32();
      let eye: Vec3 = camera.position;
      let forward: Vec3 = -camera.view.inverse().z_axis.truncate();
      let mut in_view: Vec<InView> = description
        .lights
        .iter()
        .enumerate()
        .filter(|(_, light)| settings.is_level_lights || !light.is_level)
        .filter_map(|(index, light)| {
          let basis: LightBasis = LightBasis::of(light);
          let bound: Vec4 = basis.get_bound(light);
          let is_visible: bool = planes
            .iter()
            .all(|plane| plane.truncate().dot(bound.truncate()) + plane.w >= -bound.w);

          if !is_visible {
            return None;
          }

          let fades: Fades = to_fades(light, &basis, eye, lod, is_shadowing && light.is_shadowed);

          // `light::get_LOD`: a shadowed light is drawn at all only past `EPS_L`.
          (fades.shown > EPS_L).then(|| InView {
            distance: (eye.distance(bound.truncate()) - bound.w).max(0.0),
            index,
            basis,
            bound,
            fades,
          })
        })
        .collect();

      in_view.sort_by(|a, b| a.distance.total_cmp(&b.distance));
      self.report.excess = in_view.len().saturating_sub(MAX_LIGHTS) as u32;
      in_view.truncate(MAX_LIGHTS);

      if is_shadowing {
        // Only the nearest the records could hold ask for faces.
        for it in in_view.iter().filter(|it| description.lights[it.index].is_shadowed) {
          let light: &LightDescription = &description.lights[it.index];

          self.shadows.ask(
            it.index,
            light,
            &it.basis,
            eye,
            forward,
            Vec3::from(light.color),
            contents,
            sway,
          );
        }

        self.shadows.finish();
      }

      for it in in_view {
        let light: &LightDescription = &description.lights[it.index];
        let set: Option<&LightShadowSet> = if is_shadowing && light.is_shadowed {
          // The engine never lights a shadowed light without its map: it waits for its faces.
          match self.shadows.get_set(it.index) {
            Some(set) => Some(set),
            None => continue,
          }
        } else {
          None
        };
        let color: Vec3 = to_color(description, light, seconds) * it.fades.whole;
        let range: f32 = to_frame_range(light, &mut self.random) * FALLOFF_RANGE;
        let mut record: LightRecord =
          to_record(&self.projectors, light, &it.basis, it.bound, color, range, camera.view);

        if let Some(set) = set {
          let atlas: f32 = LIGHT_SHADOW_ATLAS_SIZE as f32;

          self.report.shadowed += 1;

          record.shadow = Vec4::new(set.near, set.far, set.faces.len() as f32, 0.0);

          for (face, state) in set.faces.iter().enumerate() {
            record.faces[face] = Vec4::new(
              state.tile.x as f32 / atlas,
              state.tile.y as f32 / atlas,
              state.tile.size as f32 / atlas,
              it.fades.faces[face],
            );
          }
        }

        self.records.push(record);
      }
    }

    self.count = self.records.len() as u32;
    self.report.in_view = self.count;
    self.report.atlas = self.shadows.get_atlas_use();

    let (near, far): (f32, f32) = camera.get_depth_range();

    queue.write_buffer(
      &self.uniform,
      0,
      bytemuck::bytes_of(&LightsUniform::new(
        self.count,
        camera.projection,
        near,
        far,
        settings.shadow_filter,
      )),
    );

    if !self.records.is_empty() {
      queue.write_buffer(&self.record_buffer, 0, bytemuck::cast_slice(&self.records));
    }
  }

  /// Draws the shadow faces queued this frame, before the lights read them.
  pub fn record_shadows(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    passes: LevelPasses<'_>,
    frame: &ShadowFrame<'_>,
  ) {
    self.shadows.record(device, queue, encoder, passes, frame);
  }

  /// Clears the binning's overflow words before it counts this frame's.
  pub fn clear_overflow(&self, encoder: &mut wgpu::CommandEncoder) {
    encoder.clear_buffer(&self.counts, LIGHT_CLUSTERS * 4, Some(OVERFLOW_BYTES));
  }

  /// Copies the binning's overflow out with this frame's work, for a report a frame or more later.
  pub fn record_overflow(&self, encoder: &mut wgpu::CommandEncoder) {
    self.overflow.record(encoder, &self.counts, LIGHT_CLUSTERS * 4);
  }

  /// Asks for the overflow recorded with the frame just submitted.
  pub fn request_report(&self) {
    self.overflow.request();
  }

  /// What the last frame's lights came to, with the binning's overflow as last read back.
  pub fn take_report(&mut self) -> RenderLightsReport {
    let [full_clusters, dropped, ..] = self.overflow.take();

    RenderLightsReport {
      full_clusters,
      dropped,
      ..self.report
    }
  }

  pub fn get_shadow_atlas(&self) -> &wgpu::TextureView {
    self.shadows.get_atlas()
  }
}

/// A light standing in view this frame.
struct InView {
  /// Metres from the eye to its bound's edge, which the nearest are kept by.
  distance: f32,
  index: usize,
  basis: LightBasis,
  bound: Vec4,
  fades: Fades,
}

/// How far a light has faded: whole, as a shadowed spot fades, and each face, as a shadowed point's omni parts each
/// fade on their own, the most any of it shows.
struct Fades {
  whole: f32,
  faces: [f32; 6],
  shown: f32,
}

fn to_fades(
  light: &LightDescription,
  basis: &LightBasis,
  eye: Vec3,
  (start, end): (f32, f32),
  is_shadowed: bool,
) -> Fades {
  if !is_shadowed {
    return Fades {
      whole: 1.0,
      faces: [1.0; 6],
      shown: 1.0,
    };
  }

  if matches!(light.kind, LightKind::Spot) {
    let fade: f32 = to_light_lod(basis.get_spatial_sphere(light), eye, start, end);

    return Fades {
      whole: fade,
      faces: [1.0; 6],
      shown: fade,
    };
  }

  let faces: [f32; 6] = LIGHT_SHADOW_POINT_FACES
    .map(|(direction, _)| to_light_lod(basis.get_face_sphere(light, direction), eye, start, end));

  Fades {
    whole: 1.0,
    faces,
    shown: faces.into_iter().fold(0.0, f32::max),
  }
}

/// A light's record, in view space.
fn to_record(
  projectors: &[u32],
  light: &LightDescription,
  basis: &LightBasis,
  bound: Vec4,
  color: Vec3,
  range: f32,
  view: Mat4,
) -> LightRecord {
  let is_spot: bool = matches!(light.kind, LightKind::Spot);
  let projector: f32 = light
    .projector
    .and_then(|projector| projectors.get(projector as usize))
    .filter(|slot| **slot != MISSING_SLOT)
    .map_or(LIGHT_NO_PROJECTOR, |slot| *slot as f32);

  LightRecord {
    position: view
      .transform_point3(basis.position)
      .extend(if range > 0.0 { 1.0 / (range * range) } else { 0.0 }),
    color: color.extend(to_light_specular(color)),
    axis: view.transform_vector3(basis.direction).extend(if is_spot {
      (light.cone / 2.0).cos()
    } else {
      LIGHT_NO_CONE
    }),
    right: view.transform_vector3(basis.right).extend(if is_spot {
      to_light_shadow_scale(light.cone)
    } else {
      0.0
    }),
    up: view
      .transform_vector3(basis.up)
      .extend(if is_spot { projector } else { LIGHT_NO_PROJECTOR }),
    sphere: view.transform_point3(bound.truncate()).extend(bound.w),
    shadow: Vec4::ZERO,
    faces: [Vec4::ZERO; 6],
  }
}

/// Its colour this frame: animated where it names an animation.
fn to_color(description: &LightsDescription, light: &LightDescription, seconds: f32) -> Vec3 {
  match light
    .animator
    .and_then(|animator| description.animators.get(animator as usize))
  {
    Some(animator) => to_animated_color(animator, seconds) * light.animator_scale,
    None => Vec3::from(light.color),
  }
}

/// `UpdateIdleLight`: a light's range this frame, strayed at random by its jitter.
fn to_frame_range(light: &LightDescription, random: &mut u64) -> f32 {
  if light.range_jitter == 0.0 {
    return light.range;
  }

  // xorshift64, which needs nothing seeded but a state that is never zero.
  *random ^= *random << 13;
  *random ^= *random >> 7;
  *random ^= *random << 17;

  let unit: f32 = (*random >> 40) as f32 / (1u64 << 24) as f32;

  light.range + light.range_jitter * (unit * 2.0 - 1.0)
}
