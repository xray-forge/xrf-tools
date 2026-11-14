use xrf_level::LevelLightRole;

use crate::light::light_color::LightColor;
use crate::light::light_lamp_rays::LightLampRays;
use crate::light::light_point::LightPoint;
use crate::light::light_ray_group::LightRayGroup;
use crate::light::light_sample_point::LightSamplePoint;
use crate::light::light_source::LightSource;
use crate::light::light_term::LightTerm;

/// What lighting a few points asks of the ray model, [`LightPoint`] split in two: the shadow rays described rather than
/// made, and the light each lets through turned into each light's amount as `LightPoint` turns it. Planned and finished
/// on the CPU; its rays made and traced by either backend, numbered in the order its groups list them: a light's rays
/// one after another, a ray for each point it gives anything, in the points' order.
#[derive(Clone, Debug)]
pub(crate) struct LightPlan {
  pub(crate) light_point: LightPoint,
  /// Each role's lights the points take, by index among the lights of that role.
  pub(crate) selected: [Vec<u32>; 3],
  /// The number of each role's selected light's first ray.
  firsts: [Vec<u32>; 3],
  pub(crate) points: Vec<LightSamplePoint>,
  /// Each point's own face in the ray model, which its rays pass through; none for none. Only the GPU's batches read
  /// it: the CPU's lightmap lighter passes faces through `LightPoint` itself.
  #[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read by the GPU backend alone"))]
  pub(crate) skips: Vec<Option<u32>>,
  pub(crate) groups: Vec<LightRayGroup>,
  /// How many rays the groups make.
  pub(crate) ray_count: usize,
}

impl LightPlan {
  /// The most points a plan holds, a packet's facing bits.
  pub(crate) const MOST_POINTS: usize = LightRayGroup::FACING_WORDS * 32;

  /// The plan lighting `points`, at most [`Self::MOST_POINTS`], each passing its face of `skips`, by `light_point`
  /// from the `selected` of `lights`, each role's in [`LevelLightRole::ALL`]'s order: a light at a time, a directional
  /// light's rays as a packet, a light at a position's as `lamps` says.
  pub(crate) fn new(
    light_point: LightPoint,
    points: Vec<LightSamplePoint>,
    skips: Vec<Option<u32>>,
    lights: &[Vec<LightSource>; 3],
    selected: [Vec<u32>; 3],
    lamps: LightLampRays,
  ) -> Self {
    debug_assert!(
      points.len() <= Self::MOST_POINTS,
      "a plan's points fit a packet's facing bits"
    );

    let mut plan: Self = Self {
      light_point,
      selected,
      firsts: [Vec::new(), Vec::new(), Vec::new()],
      points,
      skips,
      groups: Vec::new(),
      ray_count: 0,
    };

    for (role, lights) in lights.iter().enumerate() {
      for index in 0..plan.selected[role].len() {
        let light: &LightSource = &lights[plan.selected[role][index] as usize];

        plan.add_light(role, light, lamps);
      }
    }

    plan
  }

  /// What each selected light gives each point, by what the plan's rays let through, `transmittances` by their
  /// numbers: each role's in turn, a light's amounts for every point in turn, `LightPoint`'s terms made again.
  pub(crate) fn get_amounts(&self, lights: &[Vec<LightSource>; 3], transmittances: &[f32]) -> [Vec<f32>; 3] {
    std::array::from_fn(|role| {
      let mut amounts: Vec<f32> = Vec::with_capacity(self.selected[role].len() * self.points.len());

      for (source, first) in self.selected[role].iter().zip(&self.firsts[role]) {
        let light: &LightSource = &lights[role][*source as usize];
        let mut ray: u32 = *first;

        for point in &self.points {
          amounts.push(
            match self.light_point.get_ray(LevelLightRole::ALL[role], light, point) {
              Some((_, term)) => {
                let amount: f32 = LightTerm { ray, ..term }.get_amount(transmittances);

                ray += 1;

                amount
              }
              None => 0.0,
            },
          );
        }
      }

      amounts
    })
  }

  /// Each point's light by what the plan's rays let through, `transmittances` by their numbers, each role's lights
  /// summed in turn as `LightPoint` sums them.
  #[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read by the GPU backend alone"))]
  pub(crate) fn get_colors(&self, lights: &[Vec<LightSource>; 3], transmittances: &[f32]) -> Vec<LightColor> {
    let amounts: [Vec<f32>; 3] = self.get_amounts(lights, transmittances);
    let point_count: usize = self.points.len();

    (0..point_count)
      .map(|index| {
        let mut color: LightColor = LightColor::default();

        for (role, selected) in self.selected.iter().enumerate() {
          for (light, source) in selected.iter().enumerate() {
            let amount: f32 = amounts[role][light * point_count + index];

            match LevelLightRole::ALL[role] {
              LevelLightRole::Rgb => color.rgb += lights[role][*source as usize].diffuse * amount,
              LevelLightRole::Hemi => color.hemi += amount,
              LevelLightRole::Sun => color.sun += amount,
            }
          }
        }

        color
      })
      .collect()
  }

  /// The rays `light` of the role at `role_index` in [`LevelLightRole::ALL`] asks, one for each point it gives
  /// anything ([`LightPoint::get_ray`]): a directional light's sharing its direction, as one packet; a light at a
  /// position's as `lamps` says.
  fn add_light(&mut self, role_index: usize, light: &LightSource, lamps: LightLampRays) {
    let role: LevelLightRole = LevelLightRole::ALL[role_index];
    let first: usize = self.ray_count;
    let mut facing: [u32; LightRayGroup::FACING_WORDS] = [0; LightRayGroup::FACING_WORDS];

    self.firsts[role_index].push(first as u32);

    for (index, point) in self.points.iter().enumerate() {
      let Some((ray, _)) = self.light_point.get_ray(role, light, point) else {
        continue;
      };

      if !light.is_directional && lamps == LightLampRays::Single {
        self.groups.push(LightRayGroup::Single {
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
      self.groups.push(LightRayGroup::Packet {
        toward: -light.direction,
        lift: match role {
          LevelLightRole::Hemi => LightPoint::HEMI_LIFT,
          LevelLightRole::Rgb | LevelLightRole::Sun => 0.0,
        },
        facing,
        first: first as u32,
      });
    } else if lamps == LightLampRays::Packet {
      self.groups.push(LightRayGroup::Lamp {
        position: light.position,
        facing,
        first: first as u32,
      });
    }
  }
}
