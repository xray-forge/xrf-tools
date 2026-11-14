use glam::Vec3;
use xrf_geometry::{BvhTracer, Facing, Ray, Tracer, Triangle};
use xrf_level::{DETAIL_SLOT_METERS, LevelDetailsHeader, LevelDetailsLight, LevelDetailsSlot, LevelLightRole};
use xrf_math::EPS;

use crate::detail::detail_slot_plan::{DetailRayGroup, DetailSlotPlan};
use crate::light::light_color::LightColor;
use crate::light::light_point::LightPoint;
use crate::light::light_point_variant::LightPointVariant;
use crate::light::light_sample_point::LightSamplePoint;
use crate::light::light_source::LightSource;
use crate::light::light_term::LightTerm;
use crate::shadow::shadow_materials::ShadowMaterials;

/// Lights one planted slot as xrDO_Light does (`LightThread::Execute` and its `LightPoint`,
/// `xrDO_Light/LightThread.cpp:174`), its arithmetic in the same order, every shadow ray a full query of the ray model
/// with no remembered blocker: planned ([`Self::plan`]), its rays traced, then finished ([`Self::finish`]).
pub(crate) struct DetailSlotLighter<'a> {
  pub(crate) header: &'a LevelDetailsHeader,
  pub(crate) tracer: &'a BvhTracer,
  pub(crate) shadows: &'a ShadowMaterials,
  /// The lights of each role, in [`LevelLightRole::ALL`]'s order.
  pub(crate) lights: &'a [Vec<LightSource>; 3],
}

impl DetailSlotLighter<'_> {
  /// xrDO_Light's `LightPoint`.
  const LIGHT_POINT: LightPoint = LightPoint {
    variant: LightPointVariant::Details,
  };

  /// `get_slot_box`'s growth of the slot's box.
  const BOX_GROW: f32 = 0.05;

  /// The further growth of the box's half extents for the triangle query.
  const QUERY_GROW: f32 = 0.01;

  /// `LIGHT_Count`: sample points run from minus to plus this many steps each way, a step being a seventh of half a
  /// slot.
  const SAMPLE_STEPS: i32 = 7;

  /// How far under the box a drop ray's height starts, so a point whose ray meets nothing stays out of it.
  const DROP_FLOOR: f32 = 5.0;

  /// What lighting the slot in grid cell `cell` asks of the ray model, or `None` where its box touches no face of it.
  pub(crate) fn plan(&self, cell: usize, slot: &LevelDetailsSlot) -> Option<DetailSlotPlan> {
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

    let points: Vec<LightSamplePoint> = Self::get_points(&triangles, center, low, high);
    let mut plan: DetailSlotPlan = DetailSlotPlan {
      selected: std::array::from_fn(|role| {
        (0..self.lights[role].len() as u32)
          .filter(|light| self.lights[role][*light as usize].reaches(sphere_center, sphere_radius))
          .collect()
      }),
      terms: [Vec::new(), Vec::new(), Vec::new()],
      origins: points.iter().map(|point| point.lifted).collect(),
      groups: Vec::new(),
      ray_count: 0,
    };

    // Each role's lights' terms, a light's for every point in turn, planned a light at a time so a directional light's
    // rays trace together.
    for (role, lights) in self.lights.iter().enumerate() {
      for index in 0..plan.selected[role].len() {
        let light: &LightSource = &lights[plan.selected[role][index] as usize];

        Self::plan_light(&mut plan, role, light, &points);
      }
    }

    Some(plan)
  }

  /// How much light each of a plan's rays lets through, traced on the CPU: the product of every face's transmittance it
  /// crosses from the front (`rayTrace`).
  pub(crate) fn trace(&self, plan: &DetailSlotPlan) -> Vec<f32> {
    let mut transmittances: Vec<f32> = vec![1.0; plan.ray_count];

    for group in &plan.groups {
      match group {
        DetailRayGroup::Packet {
          toward,
          lift,
          facing,
          first,
        } => {
          let first: usize = *first as usize;
          let rays: Vec<Ray> = DetailRayGroup::make_packet(&plan.origins, *toward, *lift, facing);

          self.tracer.for_each_packet_hit(&rays, |ray, hit| {
            transmittances[first + ray] *= self.shadows.get_transmittance(&hit);
          });
        }
        DetailRayGroup::Single { ray, index } => {
          self.tracer.for_each_hit(ray, |hit| {
            transmittances[*index as usize] *= self.shadows.get_transmittance(&hit);
          });
        }
      }
    }

    transmittances
  }

  /// The slot's light from what each of its plan's rays lets through, summed in the SDK's order: point by point, each
  /// point's lights in turn.
  pub(crate) fn finish(&self, plan: &DetailSlotPlan, transmittances: &[f32]) -> LevelDetailsLight {
    let mut sum: LightColor = LightColor::default();
    let point_count: usize = plan.origins.len();

    for index in 0..point_count {
      for (role, lights) in plan.selected.iter().enumerate() {
        for (light, source) in lights.iter().enumerate() {
          let amount: f32 = plan.terms[role][light * point_count + index].get_amount(transmittances);

          // Each sum is rounded to a float as it is stored, as the SDK's `base_color` members are: wider sums move
          // more slots across a step than they settle.
          match LevelLightRole::ALL[role] {
            LevelLightRole::Rgb => sum.rgb += self.lights[role][*source as usize].diffuse * amount,
            LevelLightRole::Hemi => sum.hemi += amount,
            LevelLightRole::Sun => sum.sun += amount,
          }
        }
      }
    }

    // No point kept leaves 0 times infinity, which quantizes to 0 as the engine's `iFloor` floors it.
    let scale: f32 = 1.0 / point_count as f32;

    sum.rgb *= scale;
    sum.hemi *= scale;
    sum.sun *= scale;
    sum.rgb *= 0.5;
    sum.hemi *= 0.5;
    sum.sun *= 0.5;

    LevelDetailsLight {
      sun: LevelDetailsLight::quantize(sum.sun),
      hemi: LevelDetailsLight::quantize(sum.hemi),
      red: LevelDetailsLight::quantize(sum.rgb.x),
      green: LevelDetailsLight::quantize(sum.rgb.y),
      blue: LevelDetailsLight::quantize(sum.rgb.z),
    }
  }

  /// The slot's 15 by 15 points across the box from `low` to `high` about `center`, each dropped onto the highest
  /// face of `triangles` under it, a later face replacing it only from more than `EPS` above; those meeting none
  /// inside the box are left out.
  fn get_points(triangles: &[Triangle], center: Vec3, low: Vec3, high: Vec3) -> Vec<LightSamplePoint> {
    let step: f32 = (DETAIL_SLOT_METERS * 0.5) / Self::SAMPLE_STEPS as f32;
    let mut points: Vec<LightSamplePoint> = Vec::new();

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
          points.push(LightSamplePoint::new(point, normal));
        }
      }
    }

    points
  }

  /// What `light` of the role at `role_index` in [`LevelLightRole::ALL`] gives each of `points`, as terms of `plan`
  /// with their rays ([`LightPoint::get_ray`]). A directional light's rays share its direction, and form one packet.
  fn plan_light(plan: &mut DetailSlotPlan, role_index: usize, light: &LightSource, points: &[LightSamplePoint]) {
    let role: LevelLightRole = LevelLightRole::ALL[role_index];
    let terms: &mut Vec<LightTerm> = &mut plan.terms[role_index];

    if !light.is_directional {
      for point in points {
        let term: LightTerm = match Self::LIGHT_POINT.get_ray(role, light, point) {
          Some((ray, term)) => {
            let index: u32 = plan.ray_count as u32;

            plan.groups.push(DetailRayGroup::Single { ray, index });
            plan.ray_count += 1;

            LightTerm { ray: index, ..term }
          }
          None => LightTerm::NOTHING,
        };

        terms.push(term);
      }

      return;
    }

    let first: usize = plan.ray_count;
    let mut facing: [u32; DetailRayGroup::FACING_WORDS] = [0; DetailRayGroup::FACING_WORDS];

    for (index, point) in points.iter().enumerate() {
      // The packet makes the ray `get_ray` describes, for each point the light gives anything.
      let Some((_, term)) = Self::LIGHT_POINT.get_ray(role, light, point) else {
        terms.push(LightTerm::NOTHING);
        continue;
      };

      facing[index / 32] |= 1 << (index % 32);
      terms.push(LightTerm {
        ray: plan.ray_count as u32,
        ..term
      });
      plan.ray_count += 1;
    }

    if plan.ray_count > first {
      plan.groups.push(DetailRayGroup::Packet {
        toward: -light.direction,
        lift: match role {
          LevelLightRole::Hemi => LightPoint::HEMI_LIFT,
          LevelLightRole::Rgb | LevelLightRole::Sun => 0.0,
        },
        facing,
        first: first as u32,
      });
    }
  }
}
