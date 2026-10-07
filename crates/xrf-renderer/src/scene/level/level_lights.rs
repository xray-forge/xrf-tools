use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3, Vec4};
use xrf_math::{EPS_L, Vector3d};
use xrf_renderer_core::{FrameGraph, GraphBindings, ProxyHandle, ProxyStore};
use xrf_visual::{LightAnimatorDescription, LightDescription, LightKind, LightsDescription};

use crate::contract::render_lights_report::RenderLightsReport;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::host::render_asset_source::RenderAssetSource;
use crate::lighting::light_animation::to_animated_color;
use crate::lighting::light_basis::{LightBasis, to_light_lod};
use crate::lighting::light_shadow_size::{LIGHT_SHADOW_POINT_FACES, to_light_shadow_scale};
use crate::lighting::light_specular::to_light_specular;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_record::{LIGHT_NO_CONE, LIGHT_NO_PROJECTOR, LightRecord};
use crate::pass::static_cull_params::StaticCullParams;
use crate::scene::level::level_light_shadows::{LIGHT_SHADOW_ATLAS_SIZE, LevelLightShadows};
use crate::scene::level::light_shadow_set::LightShadowSet;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::lights_view::{LightsView, MAX_LIGHTS};
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::zone_fast_mode::ZONE_FAST_DISTANCE;
use crate::scene::texture::texture_cache::{MISSING_SLOT, TextureCache};
use crate::scene::texture::texture_role::TextureRole;

/// What a light's falloff reaches zero at, a share of its range: `L_R` (`r3_rendertarget_accum_point.cpp`).
const FALLOFF_RANGE: f32 = 0.95;

/// A level's local lights: read once on a loader thread, moved by their motions, and each frame the nearest in a view
/// written out into its `LightsView` in its view space, animated as the engine animates them, for the lights pass to
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
  shadows: LevelLightShadows,
  started: Instant,
}

impl LevelLights {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    Self {
      lights: ProxyStore::new(),
      moving: Vec::new(),
      animators: Vec::new(),
      projectors: Vec::new(),
      shadows: LevelLightShadows::new(device, view_layout, args_size),
      started: Instant::now(),
    }
  }

  /// The texture slots the projectors sample.
  pub fn get_projectors(&self) -> &[u32] {
    &self.projectors
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
    log::info!("Native viewport lights {} local lights", lights.lights.len());
    self.animators = lights.animators;

    for light in lights.lights {
      let is_moving: bool = light.motion.is_some();
      let handle: ProxyHandle<LightDescription> = self.lights.add(light);

      if is_moving {
        self.moving.push(handle);
      }
    }
  }

  /// Writes out the lights standing in a view this frame into its own, nearest first, animated and faded as the engine
  /// would, in the camera's view space; a shadowed one only once its faces are drawn, with their squares of the atlas.
  pub fn prepare(&mut self, queue: &wgpu::Queue, into: &mut LightsView, frame: LightsFrame<'_>) {
    let LightsFrame {
      camera,
      settings,
      lod,
      contents,
      sway,
      campfire_shares,
    } = frame;

    into.records.clear();
    into.report = RenderLightsReport::default();

    let is_shadowing: bool = settings.is_enabled && settings.is_shadowed;

    if settings.is_enabled {
      let lights: &ProxyStore<LightDescription> = &self.lights;
      let planes: [Vec4; 6] = camera.get_planes();
      let seconds: f32 = self.started.elapsed().as_secs_f32();
      let eye: Vec3 = camera.position;
      let forward: Vec3 = -camera.view.inverse().z_axis.truncate();
      let mut in_view: Vec<InView> = lights
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
            handle,
            basis,
            bound,
            fades,
            share,
          })
        })
        .collect();

      in_view.sort_by(|a, b| a.distance.total_cmp(&b.distance));
      into.report.excess = in_view.len().saturating_sub(MAX_LIGHTS) as u32;
      in_view.truncate(MAX_LIGHTS);

      if is_shadowing {
        // Only the nearest the records could hold ask for faces.
        for it in in_view.iter() {
          let Some(light) = lights.get(it.handle).filter(|light| light.is_shadowed) else {
            continue;
          };

          self.shadows.ask(
            it.handle,
            light,
            &it.basis,
            eye,
            forward,
            Vec3::from(light.color),
            contents,
            sway,
          );
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
        let color: Vec3 = to_color(&self.animators, light, seconds) * it.fades.whole * it.share;
        let range: f32 = to_frame_range(light, &mut into.random) * it.share * FALLOFF_RANGE;
        let mut record: LightRecord =
          to_record(&self.projectors, light, &it.basis, it.bound, color, range, camera.view);

        if let Some(set) = set {
          let atlas: f32 = LIGHT_SHADOW_ATLAS_SIZE as f32;

          into.report.shadowed += 1;

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

        into.records.push(record);
      }
    }

    into.report.atlas = self.shadows.get_atlas_use();
    into.write(queue, camera, settings.shadow_filter);
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

  /// Ends what every view of the scene asked of the shadows this frame: picks the faces to draw from all they asked,
  /// and readies them, which `add_shadow_passes` then draws before the lights read them; then opens the next frame's
  /// asking.
  pub fn finish_shadows(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ShadowFrame<'_>,
  ) {
    self.shadows.finish();
    self.shadows.prepare(device, queue, encoder, frame);
    self.shadows.begin();
  }

  /// Declares the culls and draws of the shadow faces this frame's `prepare_shadows` readied.
  pub fn add_shadow_passes<'a>(
    &'a self,
    graph: (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    frame: (&'a StaticCullParams, &'a wgpu::BindGroup),
  ) {
    self.shadows.add_passes(graph, passes, scene, frame);
  }

  pub fn get_shadow_atlas(&self) -> &wgpu::TextureView {
    self.shadows.get_atlas()
  }
}

/// A light standing in view this frame.
struct InView {
  /// Metres from the eye to its bound's edge, which the nearest are kept by.
  distance: f32,
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
fn to_color(animators: &[LightAnimatorDescription], light: &LightDescription, seconds: f32) -> Vec3 {
  match light.animator.and_then(|animator| animators.get(animator as usize)) {
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
