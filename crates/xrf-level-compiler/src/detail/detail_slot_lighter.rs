use glam::Vec3;
use xrf_geometry::{Facing, Ray, Tracer, Triangle};
use xrf_level::{DETAIL_SLOT_METERS, LevelDetailsHeader, LevelDetailsLight, LevelDetailsSlot, LevelLightRole};
use xrf_math::EPS;

use crate::light::point::light_color::LightColor;
use crate::light::point::light_plan::LightPlan;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_roles::LightRoles;
use crate::light::source::light_sources::LightSources;
use crate::ray::ray_group::RayGroup;
use crate::ray::ray_lamp_mode::RayLampMode;
use crate::ray::ray_rules::RayRules;
use crate::ray::ray_scene::RayScene;

/// Lights one planted slot as xrDO_Light's `LightThread::Execute` does, its arithmetic in the same order, but every
/// shadow ray a full query with no remembered blocker: planned ([`Self::plan`]), traced, then finished
/// ([`Self::finish`]).
pub(crate) struct DetailSlotLighter<'a> {
  pub(crate) header: &'a LevelDetailsHeader,
  pub(crate) scene: RayScene<'a>,
  /// The lights of each role.
  pub(crate) lights: &'a LightSources,
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

  /// `LIGHT_Count`: sample points run this many steps each way, a step being a seventh of half a slot.
  const SAMPLE_STEPS: i32 = 7;

  /// How far under the box a drop ray's height starts, so a point whose ray meets nothing stays out of it.
  const DROP_FLOOR: f32 = 5.0;

  /// The rules the slots' shadow rays follow on either backend.
  pub(crate) fn get_rules() -> RayRules {
    Self::LIGHT_POINT.variant.get_rules()
  }

  /// The rays lighting the slot in grid cell `cell` needs, or `None` where its box touches no ray model face.
  pub(crate) fn plan(&self, cell: usize, slot: &LevelDetailsSlot) -> Option<LightPlan> {
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
      .scene
      .tracer
      .for_each_face_in_box(center, half + Vec3::splat(Self::QUERY_GROW), |face| {
        triangles.push(self.scene.tracer.get_triangle(face));
      });

    if triangles.is_empty() {
      return None;
    }

    let points: Vec<LightSamplePoint> = Self::get_points(&triangles, center, low, high);
    let selected: LightRoles<Vec<u32>> = self.lights.select_reaching(sphere_center, sphere_radius);
    let skips: Vec<Option<u32>> = vec![None; points.len()];

    // A slot's few rays made on the CPU, so either backend gives each slot the same light.
    Some(LightPlan::new(
      Self::LIGHT_POINT,
      points,
      skips,
      self.lights,
      selected,
      RayLampMode::Single,
    ))
  }

  /// How much light each of a plan's rays lets through, traced on the CPU: the product of the transmittances of every
  /// face it crosses from the front (`rayTrace`).
  pub(crate) fn trace(&self, plan: &LightPlan) -> Vec<f32> {
    let rules: RayRules = Self::get_rules();
    let RayScene { tracer, shadows } = self.scene;
    let mut transmittances: Vec<f32> = vec![1.0; plan.ray_count];

    for group in &plan.groups {
      match group {
        RayGroup::Directional { first, .. } | RayGroup::Lamp { first, .. } => {
          let first: usize = *first as usize;
          let rays: Vec<Ray> = plan.make_rays(group);

          tracer.for_each_packet_hit(&rays, |ray, hit| {
            transmittances[first + ray] *= rules.get_transmittance(shadows.get_opacity(&hit));
          });
        }
        RayGroup::Single { ray, index, .. } => {
          tracer.for_each_hit(ray, |hit| {
            transmittances[*index as usize] *= rules.get_transmittance(shadows.get_opacity(&hit));
          });
        }
      }
    }

    transmittances
  }

  /// The slot's light from its rays' transmittances, summed in the SDK's order: point by point, each point's lights in
  /// turn.
  pub(crate) fn finish(&self, plan: &LightPlan, transmittances: &[f32]) -> LevelDetailsLight {
    let mut sum: LightColor = LightColor::default();
    let point_count: usize = plan.points.len();
    let amounts: LightRoles<Vec<f32>> = plan.get_amounts(self.lights, transmittances);

    for index in 0..point_count {
      for (role, lights) in plan.selected.iter() {
        for (light, source) in lights.iter().enumerate() {
          let amount: f32 = amounts.get(role)[light * point_count + index];

          // Summed in single precision as the SDK's `base_color` members are: wider sums move more slots across a
          // quantization step than they settle.
          match role {
            LevelLightRole::Rgb => sum.rgb += self.lights.rgb[*source as usize].diffuse * amount,
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

  /// The slot's 15 by 15 points across the box, each dropped onto the highest face of `triangles` under it (a later
  /// face wins only from more than `EPS` above); those meeting none inside the box are left out.
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
}
