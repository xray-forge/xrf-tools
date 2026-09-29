use crate::mixer::weather_modifier::WeatherModifier;

/// Every modifier reaching a point, summed as `CEnvironment::lerp` sums them into one `CEnvModifier`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherModifiersSum {
  pub far_plane: f32,
  pub fog_color: [f32; 3],
  pub fog_density: f32,
  pub ambient: [f32; 3],
  pub sky_color: [f32; 3],
  pub hemi_color: [f32; 3],
  /// The values some modifier reaching the point adds to.
  pub flags: u16,
  /// Every reaching modifier's weight together, `mpower`.
  pub power: f32,
}

impl WeatherModifiersSum {
  /// The modifiers reaching a point, in engine space.
  pub fn at(modifiers: &[WeatherModifier], view: [f32; 3]) -> Self {
    let mut sum: Self = Self::default();

    for modifier in modifiers {
      // `sum` returns before it sets any flag for a point outside the radius.
      if modifier.is_out_of_reach(view) {
        continue;
      }

      let power: f32 = modifier.get_power(view);

      let add = |target: &mut [f32; 3], value: [f32; 3]| {
        for axis in 0..3 {
          target[axis] += value[axis] * power;
        }
      };

      if modifier.flags & WeatherModifier::FAR_PLANE != 0 {
        sum.far_plane += modifier.far_plane * power;
      }

      if modifier.flags & WeatherModifier::FOG_COLOR != 0 {
        add(&mut sum.fog_color, modifier.fog_color);
      }

      if modifier.flags & WeatherModifier::FOG_DENSITY != 0 {
        sum.fog_density += modifier.fog_density * power;
      }

      if modifier.flags & WeatherModifier::AMBIENT_COLOR != 0 {
        add(&mut sum.ambient, modifier.ambient);
      }

      if modifier.flags & WeatherModifier::SKY_COLOR != 0 {
        add(&mut sum.sky_color, modifier.sky_color);
      }

      if modifier.flags & WeatherModifier::HEMI_COLOR != 0 {
        add(&mut sum.hemi_color, modifier.hemi_color);
      }

      sum.flags |= modifier.flags;
      sum.power += power;
    }

    sum
  }

  /// `1 / (mpower + 1)`: what the environment itself is left with.
  pub fn get_scale(&self) -> f32 {
    1.0 / (self.power + 1.0)
  }

  /// Whether a value is modified at all.
  pub fn has(&self, flag: u16) -> bool {
    self.flags & flag != 0
  }
}
