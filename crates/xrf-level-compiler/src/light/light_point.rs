use glam::Vec3;
use xrf_geometry::{Ray, TraceHit, Tracer};
use xrf_level::LevelLightRole;

use crate::light::light_color::LightColor;
use crate::light::light_point_variant::LightPointVariant;
use crate::light::light_sample_point::LightSamplePoint;
use crate::light::light_source::LightSource;
use crate::light::light_term::LightTerm;
use crate::shadow::shadow_materials::ShadowMaterials;

/// The SDK's `LightPoint`: what each light gives a point with a normal, scaled by how much of it its shadow ray lets
/// through the ray model. Static lights give their colour by the cosine; the sun and the sky an amount, by the cosine
/// only when they stand at a position; a light at a position fades with its attenuation and gives nothing past its
/// range; any light gives a point it faces away from nothing, and traces no ray for it. The two compilers' copies
/// differ in a static point light's colour, which side of a face shadows and what an alpha texel lets through
/// ([`LightPointVariant`]).
///
/// Every shadow ray is a full query, where the SDK first tests the face that last blocked the light: the same result,
/// but for the order and threads the SDK's depends on. xrLC's `-att` (linear attenuation) and `LT_SECONDARY` lights,
/// which only its unported radiosity pass makes, are not followed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightPoint {
  pub variant: LightPointVariant,
}

impl LightPoint {
  /// How far a point is lifted along its normal before its shadow rays start.
  pub const POINT_LIFT: f32 = 0.01;

  /// How much further along itself a directional sky light's shadow ray starts.
  pub const HEMI_LIFT: f32 = 0.001;

  /// The range of a directional light's shadow ray.
  pub const DIRECTIONAL_RANGE: f32 = 1000.0;

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

  /// The light `point` gathers from `lights`, a role's lights in [`LevelLightRole::ALL`]'s order, of the roles
  /// `roles` names, every ray traced on the CPU against `tracer`, each face it crosses dimming it as `shadows` says.
  pub fn evaluate<T: Tracer>(
    &self,
    point: &LightSamplePoint,
    lights: &[Vec<LightSource>; 3],
    roles: &[LevelLightRole],
    tracer: &T,
    shadows: &ShadowMaterials,
  ) -> LightColor {
    self.evaluate_by(point, lights, roles, tracer, |hit| shadows.get_opacity(hit))
  }

  /// [`Self::evaluate`], each face a ray meets stopping what `opacity` says of the hit.
  pub fn evaluate_by<T: Tracer>(
    &self,
    point: &LightSamplePoint,
    lights: &[Vec<LightSource>; 3],
    roles: &[LevelLightRole],
    tracer: &T,
    opacity: impl Fn(&TraceHit) -> f32,
  ) -> LightColor {
    let mut color: LightColor = LightColor::default();

    for role in roles {
      let index: usize = LevelLightRole::ALL.iter().position(|it| it == role).unwrap_or_default();

      for light in &lights[index] {
        let Some((ray, term)) = self.get_ray(*role, light, point) else {
          continue;
        };
        let mut transmittance: f32 = 1.0;

        tracer.for_each_hit_facing(&ray, self.variant.get_facing(), |hit| {
          transmittance *= self.variant.get_transmittance(opacity(&hit));
        });

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
