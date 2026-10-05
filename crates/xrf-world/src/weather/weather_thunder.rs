use std::f32::consts::{PI, TAU};

use glam::Vec3;
use xrf_environment::WeatherMix;
use xrf_math::{EPS_L, EPS_S};
use xrf_renderer::{
  RenderThunder, RenderThunderSettings, RenderThunderbolt, RenderThunderboltGlow, RenderThunderboltStrike,
  WeatherRandom, to_interpolated_color,
};

use crate::weather::thunder_flash::ThunderFlash;

/// `CEffect_Thunderbolt::MAX_DIST_FACTOR`: the farthest a bolt strikes, of the far plane.
const MAX_DISTANCE: f32 = 0.95;

/// `EPS_L`: real seconds a strike followed at once waits after the last one ends.
const SECOND_DELAY: f32 = EPS_L;

/// The longest real step a strike takes at once, so a view shown again after a while does not skip one.
const LONGEST_STEP: f32 = 1.0;

/// A strike under way, in engine space.
struct Strike {
  name: String,
  bolt: RenderThunderbolt,
  /// Real seconds it lasts, and has lasted.
  life: f32,
  time: f32,
  /// `current_xform`'s rows: its axes scaled by its length.
  axes: [Vec3; 3],
  /// Where it strikes from, its top.
  position: Vec3,
  /// `lightning_center`, halfway down it.
  center: Vec3,
  /// `lightning_size`, its length down to the ground.
  size: f32,
  /// From it towards the view.
  direction: Vec3,
}

/// `CEffect_Thunderbolt`: a strike every period while the weather names a collection, a random bolt of it each time,
/// across the sky from the sun, at the far plane, down to the ground; and, while it strikes, the colour it lights the
/// frame with. It runs on real time, as the engine's global time does, so bolts strike on a paused clock too.
pub struct WeatherThunder {
  random: WeatherRandom,
  /// Whether the weather struck last frame, `bEnabled`.
  is_enabled: bool,
  /// Real seconds the next strike is due at.
  next: f32,
  strike: Option<Strike>,
  advanced_at: Option<f32>,
}

impl WeatherThunder {
  pub fn new(seed: u64) -> Self {
    Self {
      random: WeatherRandom::new(seed),
      is_enabled: false,
      next: 0.0,
      strike: None,
      advanced_at: None,
    }
  }

  /// Forgets any strike and when the next was due, as a weather taken afresh does.
  pub fn reset(&mut self) {
    self.is_enabled = false;
    self.strike = None;
    self.advanced_at = None;
  }

  /// `OnFrame`: schedules a strike as the weather starts to name a collection, strikes when one is due, and plays the
  /// strike under way. `now` is in real seconds and `view` in engine space; none while no strike is under way.
  pub fn advance(
    &mut self,
    thunder: &RenderThunder,
    mix: &WeatherMix,
    is_enabled: bool,
    now: f32,
    view: Vec3,
  ) -> Option<ThunderFlash> {
    let step: f32 = self.advanced_at.map_or(0.0, |at| (now - at).clamp(0.0, LONGEST_STEP));
    let palette: Vec<(&String, &RenderThunderbolt)> = if is_enabled {
      Self::list_palette(thunder, mix)
    } else {
      Vec::new()
    };
    let settings: Option<RenderThunderSettings> = thunder.settings;
    let is_striking: bool = !palette.is_empty() && settings.is_some();

    self.advanced_at = Some(now);

    if self.is_enabled != is_striking {
      let period: f32 = mix.thunderbolt_period;

      self.is_enabled = is_striking;
      self.next = now + period + self.random.between(-period * 0.5, period * 0.5);
    } else if let Some(settings) = settings.filter(|_| is_striking && now > self.next && self.strike.is_none()) {
      self.strike = Some(self.bolt(&palette, &settings, mix, now, view));
    }

    let strike: &mut Strike = self.strike.as_mut()?;
    // The engine goes idle before it steps the time, and lights this frame all the same.
    let is_over: bool = strike.time > strike.life;

    strike.time += step;

    let flash: ThunderFlash = Self::to_flash(strike, thunder, &mut self.random);

    if is_over {
      self.strike = None;
    }

    Some(flash)
  }

  /// `Bolt`: where the strike stands and how long it lasts, and when the next is due.
  fn bolt(
    &mut self,
    palette: &[(&String, &RenderThunderbolt)],
    settings: &RenderThunderSettings,
    mix: &WeatherMix,
    now: f32,
    view: Vec3,
  ) -> Strike {
    let random: &mut WeatherRandom = &mut self.random;
    let lasting: f32 = mix.thunderbolt_duration;
    let life: f32 = lasting + random.between(-lasting * 0.5, lasting * 0.5);
    let (name, bolt) = palette[((random.next_fraction() * palette.len() as f32) as usize).min(palette.len() - 1)];
    let sun_heading: f32 = to_heading(Vec3::from(mix.sun_direction));
    let far: f32 = mix.far_plane;
    let period: f32 = mix.thunderbolt_period;
    let altitude: f32 = random.between(settings.altitude[0], settings.altitude[1]);
    let longitude: f32 = random.between(
      sun_heading - settings.delta_longitude + PI,
      sun_heading + settings.delta_longitude + PI,
    );
    let distance: f32 = random.between(far * settings.min_distance, far * MAX_DISTANCE);
    let toward: Vec3 = to_direction(longitude, altitude);
    let position: Vec3 = view + toward * distance;
    let deviation: Vec3 = Vec3::new(
      random.between(-settings.tilt, settings.tilt),
      random.between(0.0, TAU),
      random.between(-settings.tilt, settings.tilt),
    );
    // `setXYZi`, which is `setHPB(-y, -x, -z)`; the light falls down the matrix's second axis.
    let [i, j, k] = to_rotation_rows(-deviation.y, -deviation.x, -deviation.z);
    let down: Vec3 = -j;
    let size: f32 = to_ground(position, down, far * 2.0);

    self.next = if random.next_fraction() < settings.second_probability {
      now + lasting + SECOND_DELAY
    } else {
      now + period + random.between(-period * 0.3, period * 0.3)
    };

    Strike {
      name: name.clone(),
      bolt: bolt.clone(),
      life,
      time: 0.0,
      axes: [i * size, j * size, k * size],
      position,
      center: position + down * (size * 0.5),
      size,
      direction: -toward,
    }
  }

  fn to_flash(strike: &Strike, thunder: &RenderThunder, random: &mut WeatherRandom) -> ThunderFlash {
    let progress: f32 = if strike.life > 0.0 {
      strike.time / strike.life
    } else {
      1.0
    };
    let phase: f32 = (1.5 * progress).clamp(0.0, 1.0);
    let bolt: &RenderThunderbolt = &strike.bolt;
    // `CalculateRGB` with the frame rate set to the frame count: the whole animation over the strike's life.
    let color: Vec3 = bolt
      .color
      .and_then(|index| thunder.animators.get(index))
      .map_or(Vec3::ZERO, |animator| {
        to_interpolated_color(
          animator,
          (progress.rem_euclid(1.0) * animator.frame_count as f32).floor(),
        )
      });
    // The top glow's opacity lights both, as `dxThunderboltRender` has it.
    let opacity: f32 = bolt.top.opacity * phase;
    let [i, j, k] = strike.axes;
    let size: f32 = strike.size;

    ThunderFlash {
      color: (color / 255.0).clamp(Vec3::ZERO, Vec3::ONE),
      direction: to_renderer(strike.direction),
      strike: RenderThunderboltStrike {
        bolt: strike.name.clone(),
        // The model's `z` is negated into renderer space as its mesh is, so its third axis turns about too.
        axes: [to_renderer(i), to_renderer(j), -to_renderer(k)],
        position: to_renderer(strike.position),
        shift: if phase > 0.5 {
          ((random.next_fraction() * 2.0) as u32).min(1) as f32 * 0.5
        } else {
          phase * 0.5
        },
        top: RenderThunderboltGlow {
          position: to_renderer(strike.position),
          extent: [bolt.top.radius[0] * size, bolt.top.radius[1] * size],
          opacity,
        },
        center: RenderThunderboltGlow {
          position: to_renderer(strike.center),
          extent: [bolt.center.radius[0] * size, bolt.center.radius[1] * size],
          opacity,
        },
      },
    }
  }

  /// The bolts of the collection the weather names now that the game has; none for no collection.
  fn list_palette<'a>(thunder: &'a RenderThunder, mix: &WeatherMix) -> Vec<(&'a String, &'a RenderThunderbolt)> {
    mix
      .thunderbolt_collection
      .as_ref()
      .and_then(|collection| thunder.collections.get(collection))
      .map_or_else(Vec::new, |names| {
        names
          .iter()
          .filter_map(|name| thunder.bolts.get_key_value(name))
          .collect()
      })
  }
}

/// `Fvector::setHP`: the direction a heading and a pitch point along, in engine space.
fn to_direction(heading: f32, pitch: f32) -> Vec3 {
  Vec3::new(-pitch.cos() * heading.sin(), pitch.sin(), pitch.cos() * heading.cos())
}

/// `Fvector::getHP`'s heading: a direction straight up or down has none, and reads as zero.
fn to_heading(direction: Vec3) -> f32 {
  if direction.x.abs() < EPS_S && direction.z.abs() < EPS_S {
    0.0
  } else {
    (-direction.x).atan2(direction.z)
  }
}

/// `Fmatrix::setHPB`'s first three rows.
fn to_rotation_rows(h: f32, p: f32, b: f32) -> [Vec3; 3] {
  let (sh, ch, sp, cp, sb, cb) = (h.sin(), h.cos(), p.sin(), p.cos(), b.sin(), b.cos());
  let (cc, cs, sc, ss) = (ch * cb, ch * sb, sh * cb, sh * sb);

  [
    Vec3::new(cc - sp * ss, -cp * sb, sp * cs + sc),
    Vec3::new(sp * sc + cs, cp * cb, ss - sp * cc),
    Vec3::new(-cp * sh, sp, cp * ch),
  ]
}

/// `RayPick` where the level's own geometry is not at hand: how far down the bolt reaches the ground plane, `y = 0`,
/// within a range, or the range where it never does.
fn to_ground(from: Vec3, down: Vec3, range: f32) -> f32 {
  if down.y.abs() < 1e-6 {
    return range;
  }

  let distance: f32 = -from.y / down.y;

  if (0.0..=range).contains(&distance) {
    distance
  } else {
    range
  }
}

/// An engine-space vector in renderer space, its `z` negated.
fn to_renderer(vector: Vec3) -> Vec3 {
  Vec3::new(vector.x, vector.y, -vector.z)
}
