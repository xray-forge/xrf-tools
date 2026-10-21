use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_math::{EPS_L, Vector3d};
use xrf_renderer_core::{FrameGraph, GraphBindings, ProxyHandle, ProxyStore};
use xrf_visual::{LightAnimatorDescription, LightDescription, LightKind, LightsDescription};

use crate::camera::camera_frustum::CameraFrustum;
use crate::camera::camera_view::CameraView;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::host::render_asset_source::RenderAssetSource;
use crate::lighting::light_animation::to_animated_color;
use crate::lighting::light_basis::LightBasis;
use crate::lighting::light_bulb::get_bulb_radius;
use crate::lighting::light_shadow_size::{LIGHT_SHADOW_POINT_FACES, to_light_shadow_scale};
use crate::lighting::light_specular::to_light_specular;
use crate::lighting::srgb::authored_colour;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_record::LightRecord;
use crate::pass::light_volume_record::LightVolumeRecord;
use crate::pass::light_volumes_uniform::LightVolumesUniform;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::level_light_shadows::LevelLightShadows;
use crate::scene::level::light_shadow_ask::LightShadowAsk;
use crate::scene::level::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::light_shadow_set::LightShadowSet;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::zone_fast_mode::ZONE_FAST_DISTANCE;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// Metres from the eye to a shadowed light's reach within which, out of view, its faces are kept ready for a turn, and
/// how many such lights are.
const WARM_REACH: f32 = 3.0;
const WARM_MOST: usize = 8;

/// What a light's falloff reaches zero at, a share of its range: `L_R` (`r3_rendertarget_accum_point.cpp`).
const FALLOFF_RANGE: f32 = 0.95;

/// The noise breaking the shafts (`s_noise`), and the projector Monolith draws every light's shafts through
/// (`s_accum_volume`, `r4_rendertarget.cpp`), which a point's faces and a spot without its own take in OpenXRay too.
const VOLUME_NOISE: &str = "fx\\fx_noise";
const VOLUME_PROJECTOR: &str = "lights\\lights_spot01";

/// `VOLUMETRIC_SLICES`: the slices the engines' buffer holds, of which a light's shafts draw some.
const VOLUME_SLICES: u32 = 100;

/// OpenXRay's fewest slices a light's shafts draw (`_max(10, iNumSlices)`).
const VOLUME_FEWEST_SLICES: u32 = 10;

/// Monolith's `ps_ssfx_volumetric` as it ships, which forces every light's shafts on: their intensity and quality, its
/// slices a quality (`24 * fQuality`), and the most a quality draws (`24 * 5`), which its depth is stretched by.
const MONOLITH_INTENSITY: f32 = 1.0;
const MONOLITH_QUALITY: f32 = 3.0;
const MONOLITH_SLICES_PER_QUALITY: f32 = 24.0;
const MONOLITH_MOST_SLICES: f32 = 120.0;

/// Metres from the eye Monolith's shafts start fading at, and the share of their intensity a metre past it takes.
const MONOLITH_FADE_START: f32 = 20.0;
const MONOLITH_FADE_RATE: f32 = 0.01;

/// What Monolith scales an omni part's shafts by (`IntensityMod`).
const MONOLITH_PART_INTENSITY: f32 = 0.2;

/// A level's local lights: read once on a loader thread, moved by their motions, and each frame the nearest in a view
/// written out into its `LightsFeature` in its view space, animated as the engine animates them, for the lights pass to
/// bin and accumulate; and the shadow faces they cast, cached in an atlas.
pub struct LevelLights {
  /// The level's lights, each reached by the handle it was added under.
  lights: ProxyStore<LightDescription>,
  /// Those a motion carries, which move each frame.
  moving: Vec<ProxyHandle<LightDescription>>,
  /// The animations replacing lights' colours, by the index a light names.
  animators: Vec<LightAnimatorDescription>,
  /// Each projector's texture slot, by its index among the description's projectors.
  projectors: Vec<u32>,
  /// The slots of the shafts' noise and of the projector Monolith draws them through.
  volume_textures: (u32, u32),
  /// Where the level's campfires burn, by their idle lights' `x` and `z`.
  fires: Vec<Vec2>,
  shadows: LevelLightShadows,
  started: Instant,
}

impl LevelLights {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout) -> Self {
    Self {
      lights: ProxyStore::new(),
      moving: Vec::new(),
      animators: Vec::new(),
      projectors: Vec::new(),
      volume_textures: (TextureCache::MISSING_SLOT, TextureCache::MISSING_SLOT),
      fires: Vec::new(),
      shadows: LevelLightShadows::new(device, view_layout),
      started: Instant::now(),
    }
  }

  /// The texture slots the projectors sample.
  pub fn get_projectors(&self) -> &[u32] {
    &self.projectors
  }

  /// Where the level's campfires burn, lit or out, by `x` and `z`.
  pub fn get_fires(&self) -> &[Vec2] {
    &self.fires
  }

  /// Adds a level's lights, asking for every projector they name.
  pub fn add_lights(
    &mut self,
    lights: LightsDescription,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) {
    self.projectors = lights
      .projectors
      .iter()
      .map(|reference| textures.request(reference, TextureRole::Projector, source))
      .collect();
    self.volume_textures = (
      textures.request(VOLUME_NOISE, TextureRole::Projector, source),
      textures.request(VOLUME_PROJECTOR, TextureRole::Projector, source),
    );
    log::info!("Native viewport lights {} local lights", lights.lights.len());
    self.animators = lights.animators;

    for light in lights.lights {
      let is_moving: bool = light.motion.is_some();

      if light.campfire.is_some() {
        self.fires.push(Vec2::new(light.position.x, light.position.z));
      }

      let handle: ProxyHandle<LightDescription> = self.lights.add(light);

      if is_moving {
        self.moving.push(handle);
      }
    }
  }

  /// Writes out the lights standing in a view this frame into its own, nearest first, animated and faded as the engine
  /// would, in the camera's view space; a shadowed one only once its faces are drawn, with their squares of the atlas.
  pub fn prepare(&mut self, encoder: &mut FrameEncoder, into: &mut LightsFeature, frame: LightsFrame<'_>) {
    let LightsFrame {
      camera,
      settings,
      engine,
      lod,
      statics,
      sway,
      grass_reach,
      campfire_shares,
      is_scene_first,
      is_physical,
      light_scale,
    } = frame;

    into.begin(
      *settings,
      // `timers.z * 0.1`: the engine's global seconds over a hundred.
      LightVolumesUniform {
        noise: self.volume_textures.0,
        scroll: self.started.elapsed().as_secs_f32() / 100.0,
      },
    );

    let is_shadowing: bool = settings.is_enabled && settings.is_shadowed;

    if is_shadowing {
      self.shadows.cast();
    }

    if settings.is_enabled {
      let lights: &ProxyStore<LightDescription> = &self.lights;
      let seconds: f32 = self.started.elapsed().as_secs_f32();
      let eye: Vec3 = camera.position;
      let forward: Vec3 = -camera.view.inverse().z_axis.truncate();
      let mut in_view: Vec<InView> = self.collect_in_view(camera, settings, (lod, campfire_shares), is_shadowing);

      in_view.sort_by(|a, b| a.distance.total_cmp(&b.distance));

      let (mut in_view, mut ready): (Vec<InView>, Vec<InView>) = in_view.into_iter().partition(|it| it.is_seen);

      into.set_excess(in_view.len().saturating_sub(LightsFeature::MAX_LIGHTS) as u32);
      in_view.truncate(LightsFeature::MAX_LIGHTS);
      ready.truncate(WARM_MOST);

      if is_shadowing {
        // Only the nearest the records could hold ask for faces, then the nearest kept ready out of view.
        for (it, is_in_view) in in_view
          .iter()
          .map(|it| (it, true))
          .chain(ready.iter().map(|it| (it, false)))
        {
          let Some(light) = lights.get(it.handle).filter(|light| light.is_shadowed) else {
            continue;
          };

          let ask: LightShadowAsk<'_> = LightShadowAsk {
            handle: it.handle,
            light,
            basis: &it.basis,
            eye,
            forward,
            is_in_view,
            is_scene_first,
          };

          self.shadows.ask(&ask, (statics, sway, grass_reach));
        }
      }

      for it in in_view {
        let Some(light) = lights.get(it.handle) else {
          continue;
        };
        let set: Option<&LightShadowSet> = if is_shadowing && light.is_shadowed {
          // The engine never lights a shadowed light without its map: it waits for its faces.
          match self.shadows.get_set(it.handle) {
            Some(set) => Some(set),
            None => continue,
          }
        } else {
          None
        };
        // The authored colour lit as light, then faded: a fade is a share of the light, linear in either model.
        let color: Vec3 =
          authored_colour(Self::to_color(&self.animators, light, seconds), is_physical) * it.fades.whole * it.share;
        let range: f32 = Self::to_frame_range(light, || into.next_random()) * it.share * FALLOFF_RANGE;
        let mut record: LightRecord = Self::to_record(
          &self.projectors,
          light,
          (&it.basis, it.bound),
          (color, is_physical),
          range,
          camera.view,
        );

        // The colour and its specular weight at the light's scale: under the engine's light the weight is a curve over
        // the light itself (`to_light_specular`), so it is taken before the scale.
        record.color *= light_scale;

        if let Some(set) = set {
          let atlas: f32 = LevelLightShadows::ATLAS_SIZE as f32;

          record.shadow = Vec4::new(set.near, set.far, set.faces.len() as f32, record.shadow.w);
          // Only the shadowed lights the level gives shafts scatter, as the engine draws only their shafts: a campfire
          // or a wall lamp lighting the whole air about it would glow through the dark as a ball of fog.
          record.scatter.x = if Self::has_shafts(light, engine) { 1.0 } else { 0.0 };

          for (face, state) in set.faces.iter().enumerate() {
            record.faces[face] = Vec4::new(
              state.tile.x as f32 / atlas,
              state.tile.y as f32 / atlas,
              state.tile.size as f32 / atlas,
              it.fades.faces[face],
            );
          }
        }

        let Some(index) = into.push(record, set.is_some()) else {
          break;
        };

        if set.is_some()
          && settings.is_volumetric
          && let Some((volume, slices)) = Self::to_volume(
            light,
            &it.basis,
            &record,
            index,
            (eye, camera.view),
            engine,
            self.volume_textures.1,
          )
        {
          into.push_volume(volume, slices);
        }
      }
    }

    into.write(encoder, camera, (settings.shadow_filter, self.shadows.get_atlas_use()));
  }

  /// The lights a view sees this frame, by its `camera` and `settings`: in its frustum, or, out of it, a shadowed one
  /// the eye stands near, its faces kept ready for a turn; each with its fades by the cull's levels of detail (`lod`)
  /// and its campfire's share (`campfire_shares`); none a campfire or a torrid zone has put out.
  fn collect_in_view(
    &self,
    camera: &CameraView,
    settings: &RenderLightsSettings,
    (lod, campfire_shares): ((f32, f32), &HashMap<u16, f32>),
    is_shadowing: bool,
  ) -> Vec<InView> {
    let frustum: CameraFrustum = camera.get_frustum();
    let eye: Vec3 = camera.position;

    self
      .lights
      .iter()
      .filter(|(_, light)| settings.is_level_lights || !light.is_level)
      .filter_map(|(handle, light)| {
        // A campfire's idle light: out while it is, faded over a turn (`UpdateWorkload`).
        let share: f32 = light
          .campfire
          .map_or(1.0, |id| campfire_shares.get(&id).copied().unwrap_or(1.0));

        if share <= 0.0 {
          return None;
        }

        // A torrid zone's idle light: out while the camera stands far from its zone (`o_switch_2_slow`).
        if let Some(sphere) = &light.zone_sphere {
          let at: [f32; 3] = [light.position.x, light.position.y, light.position.z];

          if sphere.get_distance(at, eye.to_array()) > ZONE_FAST_DISTANCE {
            return None;
          }
        }

        let basis: LightBasis = LightBasis::of(light);
        let bound: Vec4 = basis.get_bound(light);
        let is_visible: bool = frustum.reaches(bound);

        let distance: f32 = (eye.distance(bound.truncate()) - bound.w).max(0.0);

        // Out of view, only a shadowed light the eye stands near is kept, its faces ready for a turn.
        if !is_visible && !(is_shadowing && light.is_shadowed && distance <= WARM_REACH) {
          return None;
        }

        let fades: Fades = Self::to_fades(light, &basis, eye, lod, is_shadowing && light.is_shadowed);

        // `light::get_LOD`: a shadowed light is drawn at all only past `EPS_L`.
        (fades.shown > EPS_L).then_some(InView {
          distance,
          is_seen: is_visible,
          handle,
          basis,
          bound,
          fades,
          share,
        })
      })
      .collect()
  }

  /// Starts a scene's frame: stands each light its motion carries.
  pub fn begin_frame(&mut self, motions: &HashMap<String, (Mat4, Vec3)>) {
    self.move_lights(motions);
  }

  /// Stands each light a motion carries `height` over where the motion has its zone this frame (`UpdateIdleLight`);
  /// one whose motion is still read stays where its zone spawned.
  fn move_lights(&mut self, motions: &HashMap<String, (Mat4, Vec3)>) {
    for handle in &self.moving {
      let Some(motion) = self.lights.get(*handle).and_then(|light| light.motion.clone()) else {
        continue;
      };
      let Some((transform, _)) = motions.get(&motion.name) else {
        continue;
      };
      let at: Vec3 = transform.w_axis.truncate();

      // Lights stand in renderer space, which mirrors the engine's z.
      if let Some(light) = self.lights.get_mut(*handle) {
        light.position = Vector3d::new(at.x, at.y + motion.height, -at.z);
      }
    }
  }

  /// Ends what every view of the scene asked of the shadows this frame: offers `scheduler` the faces wanting a draw.
  pub fn offer_shadows(&mut self, scheduler: &mut PeriodicScheduler) {
    self.shadows.finish(scheduler);
  }

  /// Readies the faces `scheduler` granted, which `add_shadow_passes` then draws before the lights read them; then
  /// opens the next frame's asking.
  pub fn commit_shadows(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    self.shadows.prepare(device, encoder, frame, scheduler);
    self.shadows.begin();
  }

  /// Forgets the faces this frame drew, where its frame was not submitted.
  pub fn abandon_shadows(&mut self) {
    self.shadows.abandon_frame();
  }

  /// Declares the culls and draws of the shadow faces this frame's `prepare_shadows` readied, the `grass` over those
  /// casting from it.
  pub fn add_shadow_passes<'a>(
    &'a self,
    graph: (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    frame: (&'a StaticCullUniform, &'a wgpu::BindGroup),
    grass: Option<GrassShadowCasters<'a>>,
  ) {
    self.shadows.add_passes(graph, passes, scene, frame, grass);
  }

  pub fn get_shadow_atlas(&self) -> LightShadowAtlas<'_> {
    self.shadows.get_atlas()
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
      let fade: f32 = LightBasis::to_light_lod(basis.get_spatial_sphere(light), eye, start, end);

      return Fades {
        whole: fade,
        faces: [1.0; 6],
        shown: fade,
      };
    }

    let faces: [f32; 6] = LIGHT_SHADOW_POINT_FACES
      .map(|(direction, _)| LightBasis::to_light_lod(basis.get_face_sphere(light, direction), eye, start, end));

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
    (basis, bound): (&LightBasis, Vec4),
    (color, is_physical): (Vec3, bool),
    range: f32,
    view: Mat4,
  ) -> LightRecord {
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let projector: f32 = light
      .projector
      .and_then(|projector| projectors.get(projector as usize))
      .filter(|slot| **slot != TextureCache::MISSING_SLOT)
      .map_or(LightRecord::NO_PROJECTOR, |slot| *slot as f32);

    LightRecord {
      position: view
        .transform_point3(basis.position)
        .extend(if range > 0.0 { 1.0 / (range * range) } else { 0.0 }),
      color: color.extend(to_light_specular(color, is_physical)),
      axis: view.transform_vector3(basis.direction).extend(if is_spot {
        (light.cone / 2.0).cos()
      } else {
        LightRecord::NO_CONE
      }),
      right: view.transform_vector3(basis.right).extend(if is_spot {
        to_light_shadow_scale(light.cone)
      } else {
        0.0
      }),
      up: view
        .transform_vector3(basis.up)
        .extend(if is_spot { projector } else { LightRecord::NO_PROJECTOR }),
      sphere: view.transform_point3(bound.truncate()).extend(bound.w),
      shadow: Vec4::new(0.0, 0.0, 0.0, get_bulb_radius(light).powi(2)),
      faces: [Vec4::ZERO; 6],
      scatter: Vec4::ZERO,
    }
  }

  /// Whether the level gives a light shafts as the view's engine reads its description: the ones that ask under
  /// vanilla, every one described under the extended engine, whose Monolith forces them on.
  fn has_shafts(light: &LightDescription, engine: XrayEngine) -> bool {
    light
      .volumetric
      .as_ref()
      .is_some_and(|volumetric| engine == XrayEngine::Extended || volumetric.is_asked)
  }

  /// Its colour this frame: animated where it names an animation.
  fn to_color(animators: &[LightAnimatorDescription], light: &LightDescription, seconds: f32) -> Vec3 {
    match light.animator.and_then(|animator| animators.get(animator as usize)) {
      Some(animator) => to_animated_color(animator, seconds) * light.animator_scale,
      None => Vec3::from(light.color),
    }
  }

  /// `UpdateIdleLight`: a light's range this frame, strayed by its jitter at the next of the view's `random` numbers.
  fn to_frame_range(light: &LightDescription, random: impl FnOnce() -> u64) -> f32 {
    if light.range_jitter == 0.0 {
      return light.range;
    }

    let unit: f32 = (random() >> 40) as f32 / (1u64 << 24) as f32;

    light.range + light.range_jitter * (unit * 2.0 - 1.0)
  }

  /// A shadowed light's shafts this frame, and the slices they draw, as the view's engine gives them out
  /// (`r3_rendertarget_accum_spot.cpp`, `accum_volumetric`).
  fn to_volume(
    light: &LightDescription,
    basis: &LightBasis,
    record: &LightRecord,
    index: u32,
    (eye, view): (Vec3, Mat4),
    engine: XrayEngine,
    projector: u32,
  ) -> Option<(LightVolumeRecord, u32)> {
    let volumetric = light.volumetric.as_ref().filter(|_| Self::has_shafts(light, engine))?;
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    // The slices drawn, the share of the buffer the volume's depth is stretched over, the colour's scale and the
    // distance.
    let (slices, quality, intensity, distance): (u32, f32, f32, f32) = match engine {
      XrayEngine::Vanilla => {
        let asked: u32 = ((VOLUME_SLICES as f32 * volumetric.quality) as u32).max(VOLUME_FEWEST_SLICES);
        let quality: f32 = asked as f32 / VOLUME_SLICES as f32;

        (
          VOLUME_SLICES,
          quality,
          volumetric.intensity / quality,
          volumetric.distance,
        )
      }
      XrayEngine::Extended => {
        let fade: f32 = MONOLITH_INTENSITY
          - ((eye.distance(basis.position) - MONOLITH_FADE_START) * MONOLITH_FADE_RATE).clamp(0.0, 1.0)
            * MONOLITH_INTENSITY;

        if fade <= 0.0 {
          return None;
        }

        let part: f32 = if is_spot { 1.0 } else { MONOLITH_PART_INTENSITY };
        let slices: f32 = (MONOLITH_SLICES_PER_QUALITY * MONOLITH_QUALITY).floor();

        (
          slices as u32,
          slices / MONOLITH_MOST_SLICES,
          fade * part / MONOLITH_QUALITY,
          1.0,
        )
      }
    };
    let slices: u32 = if is_spot {
      slices
    } else {
      (slices as f32 * std::f32::consts::SQRT_2).round() as u32
    };
    // The light's sphere pulled towards it by its distance, in view space: the box the slices fill.
    let sphere: Vec4 = if is_spot {
      basis.get_spatial_sphere(light)
    } else {
      basis.position.extend(light.range)
    };
    let centre: Vec3 = view.transform_point3(basis.position + (sphere.truncate() - basis.position) * distance);
    let radius: f32 = sphere.w * distance;
    let near: f32 = centre.z + radius;
    let shadow: Vec4 = record.shadow;
    let slot: f32 = match engine {
      XrayEngine::Vanilla if is_spot && record.up.w >= 0.0 => record.up.w,
      _ if projector != TextureCache::MISSING_SLOT => projector as f32,
      _ => LightRecord::NO_PROJECTOR,
    };

    if slot < 0.0 {
      return None;
    }

    Some((
      LightVolumeRecord {
        near: Vec4::new(centre.x - radius, centre.y - radius, near, slices as f32),
        far: Vec4::new(
          centre.x + radius,
          centre.y + radius,
          near - 2.0 * radius / quality,
          index as f32,
        ),
        color: (record.color.truncate() * distance * intensity).extend(record.position.w / (distance * distance)),
        frustum: Vec4::new(
          shadow.x + (shadow.y - shadow.x) * distance,
          slot,
          if is_spot { 1.0 } else { 0.0 },
          0.0,
        ),
      },
      slices,
    ))
  }
}

/// A light standing in view this frame.
struct InView {
  /// Metres from the eye to its bound's edge, which the nearest are kept by.
  distance: f32,
  /// Whether the view sees its bound; a shadowed light out of view near the eye is only kept ready.
  is_seen: bool,
  handle: ProxyHandle<LightDescription>,
  basis: LightBasis,
  bound: Vec4,
  fades: Fades,
  /// How much of it its campfire lets show, all of it for any other light.
  share: f32,
}

/// How far a light has faded: whole, as a shadowed spot fades, and each face, as a shadowed point's omni parts each
/// fade on their own, the most any of it shows.
struct Fades {
  whole: f32,
  faces: [f32; 6],
  shown: f32,
}
