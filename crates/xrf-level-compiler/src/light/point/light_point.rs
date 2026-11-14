use glam::Vec3;
use xrf_geometry::{BvhTracer, Ray, TraceHit, Tracer};
use xrf_level::LevelLightRole;

use crate::light::point::light_color::LightColor;
use crate::light::point::light_occluders::LightOccluders;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::point::light_term::LightTerm;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::ray::ray_rules::RayRules;
use crate::shadow::shadow_materials::ShadowMaterials;

/// The SDK's `LightPoint`: what each light gives a point with a normal, scaled by how much of it its shadow ray lets
/// through the ray model. Static lights give their colour by the cosine; the sun and the sky an amount, by the cosine
/// only when they stand at a position; a light at a position fades with its attenuation and gives nothing past its
/// range; any light gives a point it faces away from nothing, and traces no ray for it. The two compilers' copies
/// differ in a static point light's colour and in the rules their shadow rays follow ([`LightPointVariant`]).
///
/// [`Self::evaluate`] and [`Self::evaluate_by`] trace every shadow ray as a full query; [`Self::evaluate_remembering`]
/// first tests the face that last stopped the light whole, as the SDK does, for the same light. xrLC's `-att` (linear
/// attenuation) and `LT_SECONDARY` lights, which only its unported radiosity pass makes, are not followed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LightPoint {
  pub(crate) variant: LightPointVariant,
}

impl LightPoint {
  /// How far a point is lifted along its normal before its shadow rays start.
  pub(crate) const POINT_LIFT: f32 = 0.01;

  /// How much further along itself a directional sky light's shadow ray starts.
  pub(crate) const HEMI_LIFT: f32 = 0.001;

  /// The range of a directional light's shadow ray.
  pub(crate) const DIRECTIONAL_RANGE: f32 = 1000.0;

  /// The shadow ray `light`, of `role`, asks of `point`, and the term its result goes into, its ray number unset; none
  /// where the light gives the point nothing.
  pub(crate) fn get_ray(
    &self,
    role: LevelLightRole,
    light: &LightSource,
    point: &LightSamplePoint,
  ) -> Option<(Ray, LightTerm)> {
    if light.is_directional {
      let toward: Vec3 = -light.direction;
      let cosine: f32 = toward.dot(point.normal);

      if cosine <= 0.0 {
        return None;
      }

      let (origin, factor): (Vec3, f32) = match role {
        LevelLightRole::Rgb => (point.lifted, cosine * light.energy),
        LevelLightRole::Sun => (point.lifted, light.energy),
        // No lift leaves the origin's own floats.
        LevelLightRole::Hemi => (point.lifted + toward * Self::HEMI_LIFT, light.energy),
      };

      return Some((
        Ray::new(origin, toward, Self::DIRECTIONAL_RANGE),
        LightTerm {
          factor,
          ..LightTerm::NOTHING
        },
      ));
    }

    let distance_squared: f32 = point.point.distance_squared(light.position);

    if distance_squared > light.range_squared {
      return None;
    }

    let toward: Vec3 = Self::normalize_safe(light.position - point.point);
    let cosine: f32 = toward.dot(point.normal);

    if cosine <= 0.0 {
      return None;
    }

    let distance: f32 = distance_squared.sqrt();
    let [constant, linear, quadratic] = light.attenuation;
    let attenuation: f32 = constant + linear * distance + quadratic * distance_squared;
    let (scale, divisor): (f32, f32) = match (self.variant, role) {
      (LightPointVariant::Level, LevelLightRole::Rgb) => (1.0 / attenuation - distance * light.falloff, 1.0),
      _ => (1.0, attenuation),
    };

    Some((
      Ray::new(point.lifted, toward, distance),
      LightTerm {
        factor: cosine * light.energy,
        scale,
        divisor,
        ray: LightTerm::NO_RAY,
      },
    ))
  }

  /// The shadow ray from `point` toward a light at `position`, as [`Self::get_ray`] makes it for a light it gives
  /// anything.
  pub(crate) fn get_lamp_ray(point: &LightSamplePoint, position: Vec3) -> Ray {
    let distance_squared: f32 = point.point.distance_squared(position);

    Ray::new(
      point.lifted,
      Self::normalize_safe(position - point.point),
      distance_squared.sqrt(),
    )
  }

  /// The light `point` gathers from `lights` of the roles `roles` names, in that order, every ray traced on the CPU
  /// against `tracer`, each face it crosses dimming it as `shadows` says.
  pub(crate) fn evaluate(
    &self,
    point: &LightSamplePoint,
    lights: &LightSources,
    roles: &[LevelLightRole],
    tracer: &BvhTracer,
    shadows: &ShadowMaterials,
  ) -> LightColor {
    self.evaluate_by(point, lights, roles, tracer, |hit| shadows.get_opacity(hit))
  }

  /// [`Self::evaluate`], each face a ray meets stopping what `opacity` says of the hit.
  pub(crate) fn evaluate_by(
    &self,
    point: &LightSamplePoint,
    lights: &LightSources,
    roles: &[LevelLightRole],
    tracer: &BvhTracer,
    opacity: impl Fn(&TraceHit) -> f32,
  ) -> LightColor {
    self.gather(point, lights, roles, tracer, opacity, None)
  }

  /// [`Self::evaluate_by`], each light's ray first tested against the face that last stopped that light whole
  /// (`rayTrace`'s cached polygon): where it stops this ray whole too, the ray is traced no further, as nothing else it
  /// meets could let more through. The light is the same; only the query is spared.
  pub(crate) fn evaluate_remembering(
    &self,
    point: &LightSamplePoint,
    lights: &LightSources,
    roles: &[LevelLightRole],
    tracer: &BvhTracer,
    opacity: impl Fn(&TraceHit) -> f32,
    occluders: &mut LightOccluders,
  ) -> LightColor {
    self.gather(point, lights, roles, tracer, opacity, Some(occluders))
  }

  fn gather(
    &self,
    point: &LightSamplePoint,
    lights: &LightSources,
    roles: &[LevelLightRole],
    tracer: &BvhTracer,
    opacity: impl Fn(&TraceHit) -> f32,
    mut occluders: Option<&mut LightOccluders>,
  ) -> LightColor {
    let mut color: LightColor = LightColor::default();
    let rules: RayRules = self.variant.get_rules();

    for role in roles {
      for (number, light) in lights.get(*role).iter().enumerate() {
        let Some((ray, term)) = self.get_ray(*role, light, point) else {
          continue;
        };
        let remembered: Option<u32> = occluders.as_deref().and_then(|occluders| occluders.get(*role, number));
        let is_stopped: bool = remembered.is_some_and(|face| {
          tracer
            .get_triangle(face)
            .cross(&ray, rules.facing)
            .is_some_and(|crossing| {
              opacity(&TraceHit {
                face,
                distance: crossing.distance,
                u: crossing.u,
                v: crossing.v,
              }) >= 1.0
            })
        });

        if is_stopped {
          continue;
        }

        let mut transmittance: f32 = 1.0;
        let mut stopper: Option<u32> = None;

        tracer.for_each_hit_facing(&ray, rules.facing, |hit| {
          let stops: f32 = opacity(&hit);

          if stops >= 1.0 && stopper.is_none() {
            stopper = Some(hit.face);
          }

          transmittance *= rules.get_transmittance(stops);
        });

        if let (Some(occluders), Some(face)) = (occluders.as_deref_mut(), stopper) {
          occluders.set(*role, number, face);
        }

        let amount: f32 = LightTerm { ray: 0, ..term }.get_amount(&[transmittance]);

        match role {
          LevelLightRole::Rgb => color.rgb += light.diffuse * amount,
          LevelLightRole::Hemi => color.hemi += amount,
          LevelLightRole::Sun => color.sun += amount,
        }
      }
    }

    color
  }

  /// `Fvector::normalize_safe`: scaled by the root of the inverse squared length, left as it is when too short.
  fn normalize_safe(vector: Vec3) -> Vec3 {
    let length_squared: f32 = vector.length_squared();

    if length_squared > f32::MIN_POSITIVE {
      vector * (1.0 / length_squared).sqrt()
    } else {
      vector
    }
  }
}
