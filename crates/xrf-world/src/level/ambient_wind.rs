use glam::Vec4;
use xrf_renderer::{AmbientGust, RenderAmbientEffect, WindNoise};

/// `MAX_NOISE_FREQ`: the noise's frequency at a gust factor of one.
const MAX_NOISE_FREQUENCY: f32 = 0.03;

/// What the noise's tables are filled from on every level open.
const NOISE_SEED: u64 = 0x0057_1AD5;

/// `EPS`: how near the blast's two directions may come before their turn is taken in a straight line.
const NEAR_TURN: f32 = 0.00001;

/// The wind as `CEnvironment::OnFrame` and `CGamePersistent::WeathersUpdate` drive it between them: the strength the
/// noise gives at the playing effect's gust factor, then, over that, the blast rising from what the strength was to the
/// effect's own while its direction turns, falling back to nothing once its life is up, and nothing after the fall, until
/// the next effect starts.
pub struct AmbientWind {
  noise: WindNoise,
  gust: AmbientGust,
  /// `wind_gust_factor`.
  gust_factor: f32,
  /// `ambient_effect_wind_start`, `_in_time`, `_end` and `_out_time`, in seconds.
  start: f32,
  rise_end: f32,
  end: f32,
  fall_end: f32,
  /// `ambient_effect_wind_on`: rising or blowing, before the effect's life is up.
  is_on: bool,
  /// `wind_blast_strength_start_value` and `_stop_value`.
  strengths: (f32, f32),
  /// `wind_blast_start_time` and `wind_blast_stop_time`: the directions the blast turns between, as quaternions of no
  /// angle.
  directions: (Vec4, Vec4),
}

impl Default for AmbientWind {
  fn default() -> Self {
    Self {
      noise: WindNoise::new(NOISE_SEED),
      gust: AmbientGust::default(),
      gust_factor: 0.0,
      start: 0.0,
      rise_end: 0.0,
      end: 0.0,
      fall_end: 0.0,
      is_on: false,
      strengths: (0.0, 0.0),
      directions: (Vec4::ZERO, Vec4::ZERO),
    }
  }
}

impl AmbientWind {
  pub fn get_gust(&self) -> AmbientGust {
    self.gust
  }

  /// `CEnvironment::OnFrame`: the strength the noise gives at the gust factor of the moment, in seconds.
  pub fn blow(&mut self, time: f32) {
    let noise: f32 = self.noise.read(time, self.gust_factor * MAX_NOISE_FREQUENCY);

    self.gust.strength = (noise + 0.5).clamp(0.0, 1.0);
  }

  /// An effect starts: its gusts, and its blast rising from the strength of the moment and turning from the blast's
  /// direction, or from its own where the wind was still.
  pub fn start(&mut self, effect: &RenderAmbientEffect, time: f32) {
    let blast = &effect.wind_blast;
    let life: f32 = effect.life_time.as_secs_f32();
    // `wind_blast_direction.setHP(deg2rad(wind_blast_longitude), 0)`.
    let toward: Vec4 = Vec4::new(-blast.longitude.sin(), 0.0, blast.longitude.cos(), 0.0);

    self.gust_factor = effect.wind_gust_factor;
    self.start = time;
    self.rise_end = time + blast.in_time.as_secs_f32();
    self.end = time + life;
    self.fall_end = time + life + blast.out_time.as_secs_f32();
    self.is_on = true;
    self.strengths = (self.gust.strength, blast.strength);
    self.directions = (
      if self.gust.strength == 0.0 {
        toward
      } else {
        self.gust.direction.extend(0.0)
      },
      toward,
    );
  }

  /// The effect is stopped, or the camera indoors: no more gusts.
  pub fn calm(&mut self) {
    self.gust_factor = 0.0;
  }

  /// The blast while it rises, over the noise.
  pub fn rise(&mut self, time: f32) {
    if !self.is_on || time < self.start || time > self.rise_end {
      return;
    }

    let t: f32 = Self::get_progress(time, self.start, self.rise_end);
    let (from, to) = self.directions;

    self.gust.direction = Self::slerp(from, to, t).truncate();
    self.gust.strength = self.strengths.0 + t * (self.strengths.1 - self.strengths.0);
  }

  /// The blast once the effect's life is up: falling from where it stands to nothing, and nothing once it has fallen.
  pub fn fall(&mut self, time: f32) {
    if self.is_on && time >= self.end {
      self.strengths = (self.gust.strength, 0.0);
      self.is_on = false;
    }

    if time >= self.end && time <= self.fall_end {
      let t: f32 = Self::get_progress(time, self.end, self.fall_end);

      self.gust.strength = self.strengths.0 + t * (self.strengths.1 - self.strengths.0);
    }

    if time > self.fall_end && self.fall_end != 0.0 {
      self.gust.strength = 0.0;
    }
  }

  /// How far a moment stands between two, none where they are one.
  fn get_progress(time: f32, from: f32, to: f32) -> f32 {
    if to != from { (time - from) / (to - from) } else { 0.0 }
  }

  /// `Fquaternion::slerp`: the shorter way round, straight where the two nearly meet.
  fn slerp(from: Vec4, to: Vec4, t: f32) -> Vec4 {
    let cosine: f32 = from.dot(to);
    let (cosine, sign) = if cosine < 0.0 { (-cosine, -1.0) } else { (cosine, 1.0) };
    let (from_scale, to_scale) = if 1.0 - cosine > NEAR_TURN {
      let omega: f32 = cosine.acos();
      let inverse: f32 = 1.0 / omega.sin();

      (((1.0 - t) * omega).sin() * inverse, (t * omega).sin() * inverse)
    } else {
      (1.0 - t, t)
    };

    from * from_scale + to * (to_scale * sign)
  }
}
