use glam::Vec3;
use xrf_geometry::Ray;
use xrf_level::LevelLightRole;

use crate::light::point::light_color::LightColor;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::point::light_term::LightTerm;
use crate::light::source::light_roles::LightRoles;
use crate::light::source::light_source::LightSource;
use crate::light::source::light_sources::LightSources;
use crate::ray::ray_group::RayGroup;
use crate::ray::ray_lamp_mode::RayLampMode;

/// What lighting a few points asks of the ray model, [`LightPoint`] split in two: the shadow rays described rather than
/// made, and the light each lets through turned into each light's amount as `LightPoint` turns it. Planned and finished
/// on the CPU; its rays made and traced by either backend, numbered in the order its groups list them: a light's rays
/// one after another, a ray for each point it gives anything, in the points' order.
#[derive(Clone, Debug)]
pub(crate) struct LightPlan {
  pub(crate) light_point: LightPoint,
  /// Each role's lights the points take, by index among the lights of that role.
  pub(crate) selected: LightRoles<Vec<u32>>,
  /// The number of each role's selected light's first ray.
  firsts: LightRoles<Vec<u32>>,
  pub(crate) points: Vec<LightSamplePoint>,
  /// Each point's own face in the ray model, which its rays pass through; none for none. Only the GPU's batches read
  /// it: the CPU's lightmap lighter lets a point's own face through by the opacity it gives `LightPoint`.
  #[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read by the GPU backend alone"))]
  pub(crate) skips: Vec<Option<u32>>,
  pub(crate) groups: Vec<RayGroup>,
  /// How many rays the groups make.
  pub(crate) ray_count: usize,
}

impl LightPlan {
  /// The most points a plan holds, a packet's facing bits.
  pub(crate) const MOST_POINTS: usize = RayGroup::FACING_WORDS * 32;

  /// The plan lighting `points`, at most [`Self::MOST_POINTS`], each passing its face of `skips`, by `light_point`
  /// from the `selected` of `lights`, each role's in [`LevelLightRole::ALL`]'s order: a light at a time, a directional
  /// light's rays as a packet, a light at a position's as `lamps` says.
  pub(crate) fn new(
    light_point: LightPoint,
    points: Vec<LightSamplePoint>,
    skips: Vec<Option<u32>>,
    lights: &LightSources,
    selected: LightRoles<Vec<u32>>,
    lamps: RayLampMode,
  ) -> Self {
    debug_assert!(
      points.len() <= Self::MOST_POINTS,
      "a plan's points fit a packet's facing bits"
    );

    let mut plan: Self = Self {
      light_point,
      selected,
      firsts: LightRoles::default(),
      points,
      skips,
      groups: Vec::new(),
      ray_count: 0,
    };

    for (role, lights) in lights.iter() {
      for index in 0..plan.selected.get(role).len() {
        let light: &LightSource = &lights[plan.selected.get(role)[index] as usize];

        plan.add_light(role, light, lamps);
      }
    }

    plan
  }

  /// What each selected light gives each point, by what the plan's rays let through, `transmittances` by their
  /// numbers: each role's in turn, a light's amounts for every point in turn, `LightPoint`'s terms made again.
  pub(crate) fn get_amounts(&self, lights: &LightSources, transmittances: &[f32]) -> LightRoles<Vec<f32>> {
    LightRoles::from_fn(|role| {
      let selected: &[u32] = self.selected.get(role);
      let mut amounts: Vec<f32> = Vec::with_capacity(selected.len() * self.points.len());

      for (source, first) in selected.iter().zip(self.firsts.get(role)) {
        let light: &LightSource = &lights.get(role)[*source as usize];
        let mut ray: u32 = *first;

        for point in &self.points {
          amounts.push(match self.light_point.get_ray(role, light, point) {
            Some((_, term)) => {
              let amount: f32 = LightTerm { ray, ..term }.get_amount(transmittances);

              ray += 1;

              amount
            }
            None => 0.0,
          });
        }
      }

      amounts
    })
  }

  /// Each point's light by what the plan's rays let through, `transmittances` by their numbers, each role's lights
  /// summed in turn as `LightPoint` sums them.
  #[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read by the GPU backend alone"))]
  pub(crate) fn get_colors(&self, lights: &LightSources, transmittances: &[f32]) -> Vec<LightColor> {
    let amounts: LightRoles<Vec<f32>> = self.get_amounts(lights, transmittances);
    let point_count: usize = self.points.len();

    (0..point_count)
      .map(|index| {
        let mut color: LightColor = LightColor::default();

        for (role, selected) in self.selected.iter() {
          for (light, source) in selected.iter().enumerate() {
            let amount: f32 = amounts.get(role)[light * point_count + index];

            match role {
              LevelLightRole::Rgb => color.rgb += lights.rgb[*source as usize].diffuse * amount,
              LevelLightRole::Hemi => color.hemi += amount,
              LevelLightRole::Sun => color.sun += amount,
            }
          }
        }

        color
      })
      .collect()
  }

  /// `group`'s rays as the CPU makes them, from the plan's points, in their points' order.
  pub(crate) fn make_rays(&self, group: &RayGroup) -> Vec<Ray> {
    let facing_points = |facing: &[u32; RayGroup::FACING_WORDS]| {
      self
        .points
        .iter()
        .enumerate()
        .filter(|(point, _)| RayGroup::is_facing(facing, *point))
        .map(|(_, point)| *point)
        .collect::<Vec<LightSamplePoint>>()
    };

    match group {
      RayGroup::Packet {
        toward,
        lift,
        range,
        facing,
        ..
      } => facing_points(facing)
        .iter()
        .map(|point| {
          // No lift leaves the origin's own floats.
          let start: Vec3 = if *lift == 0.0 {
            point.lifted
          } else {
            point.lifted + *toward * *lift
          };

          Ray::new(start, *toward, *range)
        })
        .collect(),
      RayGroup::Lamp { position, facing, .. } => facing_points(facing)
        .iter()
        .map(|point| LightPoint::get_lamp_ray(point, *position))
        .collect(),
      RayGroup::Single { ray, .. } => vec![*ray],
    }
  }

  /// The rays `light` of `role` asks, one for each point it gives anything ([`LightPoint::get_ray`]): a directional
  /// light's sharing its direction, as one packet; a light at a position's as `lamps` says.
  fn add_light(&mut self, role: LevelLightRole, light: &LightSource, lamps: RayLampMode) {
    let first: usize = self.ray_count;
    let mut facing: [u32; RayGroup::FACING_WORDS] = [0; RayGroup::FACING_WORDS];

    self.firsts.get_mut(role).push(first as u32);

    for (index, point) in self.points.iter().enumerate() {
      let Some((ray, _)) = self.light_point.get_ray(role, light, point) else {
        continue;
      };

      if !light.is_directional && lamps == RayLampMode::Single {
        self.groups.push(RayGroup::Single {
          ray,
          index: self.ray_count as u32,
          point: index as u32,
        });
      }

      facing[index / 32] |= 1 << (index % 32);
      self.ray_count += 1;
    }

    if self.ray_count == first {
      return;
    }

    if light.is_directional {
      self.groups.push(RayGroup::Packet {
        toward: -light.direction,
        lift: match role {
          LevelLightRole::Hemi => LightPoint::HEMI_LIFT,
          LevelLightRole::Rgb | LevelLightRole::Sun => 0.0,
        },
        range: LightPoint::DIRECTIONAL_RANGE,
        facing,
        first: first as u32,
      });
    } else if lamps == RayLampMode::Packet {
      self.groups.push(RayGroup::Lamp {
        position: light.position,
        facing,
        first: first as u32,
      });
    }
  }
}

#[cfg(feature = "gpu")]
impl crate::ray::gpu::ray_gpu_plan::RayGpuPlan for LightPlan {
  fn get_point_count(&self) -> usize {
    self.points.len()
  }

  fn get_origin(&self, point: usize) -> Vec3 {
    self.points[point].lifted
  }

  fn get_point(&self, point: usize) -> Vec3 {
    self.points[point].point
  }

  fn get_skip(&self, point: usize) -> Option<u32> {
    self.skips[point]
  }

  fn get_groups(&self) -> &[RayGroup] {
    &self.groups
  }

  fn get_ray_count(&self) -> usize {
    self.ray_count
  }
}
