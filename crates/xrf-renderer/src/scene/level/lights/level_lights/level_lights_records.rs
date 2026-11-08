use glam::{Mat4, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_renderer_core::ProxyStore;
use xrf_visual::{LightAnimatorDescription, LightDescription, LightKind, LightVolumetric};

use crate::contract::feature::lighting::lights::render_lights_settings::RenderLightsSettings;
use crate::contract::feature::weather::render_light_shafts::RenderLightShafts;
use crate::feature::lighting::lights::lights_feature::LightsFeature;
use crate::frame::frame_encoder::FrameEncoder;
use crate::lighting::lamp::light_animation::to_animated_color;
use crate::lighting::lamp::light_basis::LightBasis;
use crate::lighting::lamp::light_bulb::get_bulb_radius;
use crate::lighting::lamp::light_shadow_size::{LIGHT_SHADOW_POINT_FACES, to_light_shadow_scale};
use crate::lighting::lamp::light_specular::to_light_specular;
use crate::lighting::srgb::{anchored_light, authored_light};
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::lights::asks::light_asks::LightAsks;
use crate::scene::level::lights::asks::light_in_view::{LightFades, LightInView};
use crate::scene::level::lights::asks::lights_frame::LightsFrame;
use crate::scene::level::lights::atlas::level_light_shadows::LevelLightShadows;
use crate::scene::level::lights::atlas::light_shadow_set::LightShadowSet;
use crate::scene::level::lights::level_lights::{
  FALLOFF_RANGE, LevelLights, LightShafts, MONOLITH_FADE_RATE, MONOLITH_FADE_START, MONOLITH_INTENSITY,
  MONOLITH_MOST_SLICES, MONOLITH_PART_INTENSITY, MONOLITH_QUALITY, MONOLITH_SLICES_PER_QUALITY, VOLUME_DEFAULTS,
  VOLUME_FEWEST_SLICES, VOLUME_SLICES,
};
use crate::scene::level::lights::light_record::LightRecord;
use crate::scene::level::lights::light_volume_record::LightVolumeRecord;
use crate::scene::level::lights::light_volumes_uniform::LightVolumesUniform;
use crate::scene::texture::texture_cache::TextureCache;

impl LevelLights {
  /// Writes out the lights a view picked into its own, nearest first, animated and faded as the engine would, in the
  /// camera's view space; a shadowed one only once its faces are drawn, with their squares of the atlas.
  pub(crate) fn write(&self, encoder: &mut FrameEncoder, into: &mut LightsFeature, asks: &LightAsks) {
    let LightsFrame {
      camera,
      settings,
      engine,
      is_physical,
      is_specular_linear,
      anchor,
      is_falloff_inverse_square,
      light_scale,
      time,
      ..
    } = asks.frame;

    into.begin(settings, LightVolumesUniform::new(self.volume_textures.0, time));

    let is_shadowing: bool = asks.frame.is_shadowing();

    if settings.is_enabled {
      let lights: &ProxyStore<LightDescription> = &self.lights;
      let eye: Vec3 = camera.position;

      into.set_excess(asks.excess);

      for it in &asks.in_view {
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
        // The authored colour faded, a fade a share of the light, linear in either model; as light, its strength kept
        // as the weather's lights keep theirs, or at the anchored levels anchored over the ambient. Its specular weight
        // follows the light at the anchored and physical levels, else the engine's over the authored colour, which the
        // authored levels add to the diffuse in display values as the engine does (`hmodel_terms`).
        let authored: Vec3 = Self::to_color(&self.animators, light, time) * it.fades.whole * it.share;
        let color: Vec3 = match anchor {
          Some(under) => anchored_light(authored, under),
          None => authored_light(authored, is_physical),
        };
        let specular: f32 = if is_specular_linear {
          to_light_specular(color, true)
        } else {
          to_light_specular(authored, false)
        };
        let jittered: f32 = Self::to_frame_range(light, || into.next_random());
        let range: f32 = jittered * it.share * FALLOFF_RANGE;
        // Under the inverse square a light's intensity follows its range squared: its strength is held to its own
        // range, so an idle light's jitter strays only the window's edge, as under the engine's falloff, rather than
        // flash the whole pool each frame by the square of its stray.
        let held: f32 = if is_falloff_inverse_square {
          Self::to_jitter_hold(light.range, jittered)
        } else {
          1.0
        };
        let mut record: LightRecord = Self::to_record(
          &self.projectors,
          light,
          it,
          color * held,
          specular * held,
          range,
          camera.view,
        );

        // The colour and its specular weight at the light's scale: under the engine's light the weight is a curve over
        // the light itself (`to_light_specular`), so it is taken before the scale.
        record.color *= light_scale;

        let shafts: Option<LightShafts> = set.and_then(|_| Self::to_shafts(light, &it.basis, eye, engine, &settings));

        if let Some(set) = set {
          let atlas: f32 = LevelLightShadows::ATLAS_SIZE as f32;

          record.shadow = Vec4::new(set.near, set.far, set.faces.len() as f32, record.shadow.w);
          // Only the shadowed lights the level gives shafts scatter, as thickly as their shafts: a campfire or a wall
          // lamp lighting the whole air about it would glow through the dark as a ball of fog.
          record.scatter.x = shafts.as_ref().map_or(0.0, LightShafts::get_density);

          for (face, state) in set.faces.iter().enumerate() {
            record.faces[face] = Vec4::new(
              state.tile.x as f32 / atlas,
              state.tile.y as f32 / atlas,
              state.tile.size as f32 / atlas,
              it.fades.faces[face],
            );
          }
        }

        let pushed: Option<u32> = if set.is_some() {
          into.push_shadowed(record)
        } else {
          into.push(record)
        };
        let Some(index) = pushed else {
          break;
        };

        if let Some(shafts) = &shafts
          && let Some((volume, slices)) = Self::to_volume(
            shafts,
            &it.basis,
            &record,
            index,
            camera.view,
            engine,
            self.volume_textures.1,
          )
        {
          into.push_volume(volume, slices);
        }
      }
    }

    into.write(encoder, &camera, settings.shadow_filter, self.shadows.get_atlas_use());
  }

  /// How far a shadowed light has faded from `eye` over the cull's levels of detail, `start` to `end` metres.
  pub(super) fn to_shadow_fades(
    light: &LightDescription,
    basis: &LightBasis,
    eye: Vec3,
    (start, end): (f32, f32),
  ) -> LightFades {
    if matches!(light.kind, LightKind::Spot) {
      let fade: f32 = LightBasis::to_light_lod(basis.get_spatial_sphere(light), eye, start, end);

      return LightFades {
        whole: fade,
        faces: [1.0; 6],
        shown: fade,
      };
    }

    let faces: [f32; 6] = LIGHT_SHADOW_POINT_FACES
      .map(|(direction, _)| LightBasis::to_light_lod(basis.get_face_sphere(light, direction), eye, start, end));

    LightFades {
      whole: 1.0,
      faces,
      shown: faces.into_iter().fold(0.0, f32::max),
    }
  }

  /// A light's record, in view space: standing as `it` sees it, of `color` and `specular` weight, out to `range`.
  fn to_record(
    projectors: &[u32],
    light: &LightDescription,
    it: &LightInView,
    color: Vec3,
    specular: f32,
    range: f32,
    view: Mat4,
  ) -> LightRecord {
    let (basis, bound): (&LightBasis, Vec4) = (&it.basis, it.bound);
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
      color: color.extend(specular),
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

  /// The shaft values a light draws by, where it draws any: as the view's engine reads its description, the ones that
  /// ask under vanilla and every described one under the extended engine, whose Monolith forces them on; or, where the
  /// view asks for every shadowed light's (`RenderLightShafts::Shadowed`), the engine's defaults for any that does not
  /// ask.
  fn to_volumetric(light: &LightDescription, engine: XrayEngine, shafts: RenderLightShafts) -> Option<LightVolumetric> {
    match &light.volumetric {
      Some(volumetric) if volumetric.is_asked || engine == XrayEngine::Extended => Some(volumetric.clone()),
      _ if shafts == RenderLightShafts::Shadowed => Some(VOLUME_DEFAULTS),
      _ => None,
    }
  }

  /// Its colour this frame: animated where it names an animation.
  fn to_color(animators: &[LightAnimatorDescription], light: &LightDescription, time: LevelTime) -> Vec3 {
    match light.animator.and_then(|animator| animators.get(animator as usize)) {
      Some(animator) => to_animated_color(animator, time) * light.animator_scale,
      None => Vec3::from(light.color),
    }
  }

  /// What a light's colour is multiplied by under the inverse square to give at its `jittered` range what it gives at
  /// its own `range`: their squares' ratio, one for a light that does not stray.
  pub(crate) fn to_jitter_hold(range: f32, jittered: f32) -> f32 {
    if jittered > 0.0 && range > 0.0 {
      (range / jittered).powi(2)
    } else {
      1.0
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

  /// The shafts a shadowed light draws this frame, seen from `eye`, as the view's engine gives them out
  /// (`r3_rendertarget_accum_spot.cpp`, `accum_volumetric`) to the lights the `settings` pick, at their strength: none
  /// where the settings turn shafts off or it is given none, or where Monolith's have faded.
  pub(crate) fn to_shafts(
    light: &LightDescription,
    basis: &LightBasis,
    eye: Vec3,
    engine: XrayEngine,
    settings: &RenderLightsSettings,
  ) -> Option<LightShafts> {
    if !settings.is_volumetric || settings.shaft_strength <= 0.0 {
      return None;
    }

    let volumetric: LightVolumetric = Self::to_volumetric(light, engine, settings.shafts)?;
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

    Some(LightShafts {
      is_spot,
      slices: if is_spot {
        slices
      } else {
        (slices as f32 * std::f32::consts::SQRT_2).round() as u32
      },
      quality,
      intensity: intensity * settings.shaft_strength,
      distance,
      sphere: if is_spot {
        basis.get_spatial_sphere(light)
      } else {
        basis.position.extend(light.range)
      },
    })
  }

  /// A shadowed light's `shafts` as volume `index` draws them, and its slices, in the `view`'s space.
  fn to_volume(
    shafts: &LightShafts,
    basis: &LightBasis,
    record: &LightRecord,
    index: u32,
    view: Mat4,
    engine: XrayEngine,
    projector: u32,
  ) -> Option<(LightVolumeRecord, u32)> {
    let LightShafts {
      is_spot,
      slices,
      quality,
      intensity,
      distance,
      sphere,
    } = *shafts;
    // The light's sphere pulled towards it by its distance, in view space: the box the slices fill.
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
