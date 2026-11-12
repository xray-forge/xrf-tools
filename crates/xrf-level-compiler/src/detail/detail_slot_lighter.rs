use glam::Vec3;
use xrf_geometry::{BvhTracer, Facing, Ray, Tracer, Triangle};
use xrf_level::{DETAIL_SLOT_METERS, LevelDetailsHeader, LevelDetailsLight, LevelDetailsSlot, LevelLightRole};
use xrf_math::EPS;

use crate::detail::detail_light_source::DetailLightSource;
use crate::shadow::shadow_materials::ShadowMaterials;

/// Lights one planted slot as xrDO_Light does (`LightThread::Execute` and `LightPoint`,
/// `xrDO_Light/LightThread.cpp:174`), its arithmetic in the same order, every shadow ray a full query of the ray model
/// with no remembered blocker.
pub(crate) struct DetailSlotLighter<'a> {
  pub(crate) header: &'a LevelDetailsHeader,
  pub(crate) tracer: &'a BvhTracer,
  pub(crate) shadows: &'a ShadowMaterials,
  /// The lights of each role, in [`LevelLightRole::ALL`]'s order.
  pub(crate) lights: &'a [Vec<DetailLightSource>; 3],
}

/// The light a slot's points gather, `base_color` (`xrDO_Light/LightThread.cpp:26`).
#[derive(Clone, Copy, Debug, Default)]
struct DetailLightSum {
  rgb: Vec3,
  hemi: f32,
  sun: f32,
}

/// A point of a slot the drop ray kept: where it lies, its ground's normal, and where its shadow rays start.
#[derive(Clone, Copy, Debug)]
struct DetailSamplePoint {
  point: Vec3,
  normal: Vec3,
  lifted: Vec3,
}

impl DetailSlotLighter<'_> {
  /// `get_slot_box`'s growth of the slot's box.
  const BOX_GROW: f32 = 0.05;

  /// The further growth of the box's half extents for the triangle query.
  const QUERY_GROW: f32 = 0.01;

  /// `LIGHT_Count`: sample points run from minus to plus this many steps each way, a step being a seventh of half a
  /// slot.
  const SAMPLE_STEPS: i32 = 7;

  /// How far under the box a drop ray's height starts, so a point whose ray meets nothing stays out of it.
  const DROP_FLOOR: f32 = 5.0;

  /// How far a point is lifted along its ground's normal before its shadow rays start.
  const POINT_LIFT: f32 = 0.01;

  /// How much further along itself a directional hemisphere light's shadow ray starts.
  const HEMI_LIFT: f32 = 0.001;

  /// The range of a directional light's shadow ray.
  const DIRECTIONAL_RANGE: f32 = 1000.0;

  /// The light of the slot in grid cell `cell`, or `None` where its box touches no face of the ray model.
  pub(crate) fn light(&self, cell: usize, slot: &LevelDetailsSlot) -> Option<LevelDetailsLight> {
    // A grid's sides are `u32`, so a cell's column and row are too.
    let column: u32 = (cell % self.header.size_x as usize) as u32;
    let row: u32 = (cell / self.header.size_x as usize) as u32;
    let minimum: Vec3 = Vec3::new(
      (i64::from(column) - i64::from(self.header.offset_x)) as f32 * DETAIL_SLOT_METERS,
      slot.base_height,
      (i64::from(row) - i64::from(self.header.offset_z)) as f32 * DETAIL_SLOT_METERS,
    );
    let maximum: Vec3 = minimum + Vec3::new(DETAIL_SLOT_METERS, slot.height, DETAIL_SLOT_METERS);
    let low: Vec3 = minimum - Vec3::splat(Self::BOX_GROW);
    let high: Vec3 = maximum + Vec3::splat(Self::BOX_GROW);
    // `Fbox::get_CD` for the query and the samples, `Fbox::getsphere` for the lights: two centres, computed apart.
    let half: Vec3 = (high - low) * 0.5;
    let center: Vec3 = low + half;
    let sphere_center: Vec3 = (low + high) * 0.5;
    let sphere_radius: f32 = sphere_center.distance(high);
    let mut triangles: Vec<Triangle> = Vec::new();

    self
      .tracer
      .for_each_face_in_box(center, half + Vec3::splat(Self::QUERY_GROW), |face| {
        triangles.push(self.tracer.get_triangle(face));
      });

    if triangles.is_empty() {
      return None;
    }

    let points: Vec<DetailSamplePoint> = Self::get_points(&triangles, center, low, high);
    let selected: [Vec<&DetailLightSource>; 3] = std::array::from_fn(|role| {
      self.lights[role]
        .iter()
        .filter(|light| light.reaches(sphere_center, sphere_radius))
        .collect()
    });
    // Each role's lights' amounts, a light's for every point in turn, gathered a light at a time so a directional
    // light's rays trace together, and summed below in the SDK's order: point by point, each point's lights in turn.
    let amounts: [Vec<f32>; 3] = std::array::from_fn(|role| {
      selected[role]
        .iter()
        .flat_map(|light| self.get_amounts(LevelLightRole::ALL[role], light, &points))
        .collect()
    });
    let mut sum: DetailLightSum = DetailLightSum::default();

    for index in 0..points.len() {
      for (role, lights) in selected.iter().enumerate() {
        for (light, source) in lights.iter().enumerate() {
          let amount: f32 = amounts[role][light * points.len() + index];

          // Each sum is rounded to a float as it is stored, as the SDK's `base_color` members are: wider sums move
          // more slots across a step than they settle.
          match LevelLightRole::ALL[role] {
            LevelLightRole::Rgb => sum.rgb += source.diffuse * amount,
            LevelLightRole::Hemi => sum.hemi += amount,
            LevelLightRole::Sun => sum.sun += amount,
          }
        }
      }
    }

    // No point kept leaves 0 times infinity, which quantizes to 0 as the engine's `iFloor` floors it.
    let scale: f32 = 1.0 / points.len() as f32;

    sum.rgb *= scale;
    sum.hemi *= scale;
    sum.sun *= scale;
    sum.rgb *= 0.5;
    sum.hemi *= 0.5;
    sum.sun *= 0.5;

    Some(LevelDetailsLight {
      sun: LevelDetailsLight::quantize(sum.sun),
      hemi: LevelDetailsLight::quantize(sum.hemi),
      red: LevelDetailsLight::quantize(sum.rgb.x),
      green: LevelDetailsLight::quantize(sum.rgb.y),
      blue: LevelDetailsLight::quantize(sum.rgb.z),
    })
  }

  /// The slot's 15 by 15 points across the box from `low` to `high` about `center`, each dropped onto the highest
  /// face of `triangles` under it, a later face replacing it only from more than `EPS` above; those meeting none
  /// inside the box are left out.
  fn get_points(triangles: &[Triangle], center: Vec3, low: Vec3, high: Vec3) -> Vec<DetailSamplePoint> {
    let step: f32 = (DETAIL_SLOT_METERS * 0.5) / Self::SAMPLE_STEPS as f32;
    let mut points: Vec<DetailSamplePoint> = Vec::new();

    for sample_x in -Self::SAMPLE_STEPS..=Self::SAMPLE_STEPS {
      for sample_z in -Self::SAMPLE_STEPS..=Self::SAMPLE_STEPS {
        let mut point: Vec3 = Vec3::new(
          center.x + step * sample_x as f32,
          low.y - Self::DROP_FLOOR,
          center.z + step * sample_z as f32,
        );
        let drop: Ray = Ray::new(Vec3::new(point.x, high.y + EPS, point.z), Vec3::NEG_Y, f32::MAX);
        let mut normal: Vec3 = Vec3::Y;

        for triangle in triangles {
          if let Some(crossing) = triangle.cross(&drop, Facing::Front) {
            let height: f32 = drop.origin.y - crossing.distance;

            if height > point.y {
              point.y = height + EPS;
              normal = triangle.get_normal();
            }
          }
        }

        if point.y >= low.y {
          points.push(DetailSamplePoint {
            point,
            normal,
            lifted: point + normal * Self::POINT_LIFT,
          });
        }
      }
    }

    points
  }

  /// What `light` of `role` gives each of `points` (`LightPoint`), 0 where it gives nothing: the static lights' colour
  /// by the cosine, the sun and sky as plain amounts, a point light of any role through its attenuation, each scaled
  /// by how much of it reaches the point. A directional light's rays share its direction, and trace as one packet.
  fn get_amounts(&self, role: LevelLightRole, light: &DetailLightSource, points: &[DetailSamplePoint]) -> Vec<f32> {
    if !light.is_directional {
      return points
        .iter()
        .map(|point| self.get_point_light_amount(light, point))
        .collect();
    }

    let toward: Vec3 = -light.direction;
    let cosines: Vec<f32> = points.iter().map(|point| toward.dot(point.normal)).collect();
    // The rays of the points the light faces, by point.
    let facing: Vec<usize> = (0..points.len()).filter(|index| cosines[*index] > 0.0).collect();
    let rays: Vec<Ray> = facing
      .iter()
      .map(|index| {
        let lifted: Vec3 = points[*index].lifted;
        let origin: Vec3 = match role {
          LevelLightRole::Hemi => lifted + toward * Self::HEMI_LIFT,
          LevelLightRole::Rgb | LevelLightRole::Sun => lifted,
        };

        Ray::new(origin, toward, Self::DIRECTIONAL_RANGE)
      })
      .collect();
    let mut transmittances: Vec<f32> = vec![1.0; rays.len()];

    self.tracer.for_each_packet_hit(&rays, |ray, hit| {
      transmittances[ray] *= self.shadows.get_transmittance(&hit);
    });

    let mut amounts: Vec<f32> = vec![0.0; points.len()];

    for (index, transmittance) in facing.into_iter().zip(transmittances) {
      amounts[index] = match role {
        LevelLightRole::Rgb => cosines[index] * light.energy * transmittance,
        LevelLightRole::Hemi | LevelLightRole::Sun => light.energy * transmittance,
      };
    }

    amounts
  }

  /// What a light at a position gives `point`, 0 past its range or behind the ground.
  fn get_point_light_amount(&self, light: &DetailLightSource, point: &DetailSamplePoint) -> f32 {
    let distance_squared: f32 = point.point.distance_squared(light.position);

    if distance_squared > light.range_squared {
      return 0.0;
    }

    let toward: Vec3 = Self::normalize_safe(light.position - point.point);
    let cosine: f32 = toward.dot(point.normal);

    if cosine <= 0.0 {
      return 0.0;
    }

    let distance: f32 = distance_squared.sqrt();
    let [constant, linear, quadratic] = light.attenuation;

    cosine * light.energy * self.trace(point.lifted, toward, distance)
      / (constant + linear * distance + quadratic * distance_squared)
  }

  /// How much light passes from `origin` along `direction` over `range` (`rayTrace`): the product of every face's
  /// transmittance the ray crosses from the front.
  fn trace(&self, origin: Vec3, direction: Vec3, range: f32) -> f32 {
    let mut transmittance: f32 = 1.0;

    self.tracer.for_each_hit(&Ray::new(origin, direction, range), |hit| {
      transmittance *= self.shadows.get_transmittance(&hit);
    });

    transmittance
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
