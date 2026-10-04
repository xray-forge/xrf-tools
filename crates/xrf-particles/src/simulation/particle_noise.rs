use std::sync::OnceLock;

use glam::Vec3;

use crate::simulation::particle_random::ParticleRandom;

/// `noise.cpp`: Perlin gradient noise, its tables filled from `srand(1)` as `noise3Init` fills them.
pub(crate) struct ParticleNoise {
  permutation: [usize; Self::TABLE + Self::TABLE + 2],
  gradients: [[f32; 3]; Self::TABLE + Self::TABLE + 2],
}

impl ParticleNoise {
  /// `B`, the lattice period.
  const TABLE: usize = 256;

  /// `fractalsum3`'s frequency step between octaves.
  const LACUNARITY: f32 = 2.059;

  /// The tables, built on first use.
  pub(crate) fn get() -> &'static Self {
    static NOISE: OnceLock<ParticleNoise> = OnceLock::new();

    NOISE.get_or_init(Self::new)
  }

  /// `noise3Init`.
  fn new() -> Self {
    let table: usize = Self::TABLE;
    let mut random: ParticleRandom = ParticleRandom::new(1);
    let mut permutation: [usize; Self::TABLE + Self::TABLE + 2] = [0; Self::TABLE + Self::TABLE + 2];
    let mut gradients: [[f32; 3]; Self::TABLE + Self::TABLE + 2] = [[0.0; 3]; Self::TABLE + Self::TABLE + 2];

    for gradient in gradients.iter_mut().take(table) {
      let mut vector: [f32; 3];
      let mut length_squared: f32;

      loop {
        vector = [0.0; 3];

        for component in &mut vector {
          *component = ((random.next_integer() % (2 * table as i32)) - table as i32) as f32 / table as f32;
        }

        length_squared = vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2];

        if length_squared <= 1.0 {
          break;
        }
      }

      let length: f32 = length_squared.sqrt();

      *gradient = [vector[0] / length, vector[1] / length, vector[2] / length];
    }

    for (index, slot) in permutation.iter_mut().take(table).enumerate() {
      *slot = index;
    }

    let mut index: usize = table;

    while index > 0 {
      let swapped: usize = random.next_integer() as usize % table;

      permutation.swap(index, swapped);
      index -= 2;
    }

    for index in 0..table + 2 {
      permutation[table + index] = permutation[index];
      gradients[table + index] = gradients[index];
    }

    Self { permutation, gradients }
  }

  /// `fractalsum3`: octaves of noise, each 2.059 times the last frequency and weighed by its inverse.
  pub(crate) fn fractal_sum(&self, point: Vec3, frequency: f32, octaves: i32) -> f32 {
    let boost: f32 = frequency;
    let mut frequency: f32 = frequency;
    let mut sum: f32 = 0.0;

    for _ in 0..octaves {
      sum += self.sample(point * frequency) / frequency;
      frequency *= Self::LACUNARITY;
    }

    sum * boost
  }

  /// `noise3`.
  fn sample(&self, point: Vec3) -> f32 {
    let (bx0, bx1, rx0, rx1) = Self::setup(point.x);
    let (by0, by1, ry0, ry1) = Self::setup(point.y);
    let (bz0, bz1, rz0, rz1) = Self::setup(point.z);

    let i: usize = self.permutation[bx0];
    let j: usize = self.permutation[bx1];

    let b00: usize = self.permutation[i + by0];
    let b10: usize = self.permutation[j + by0];
    let b01: usize = self.permutation[i + by1];
    let b11: usize = self.permutation[j + by1];

    let sx: f32 = Self::s_curve(rx0);
    let sy: f32 = Self::s_curve(ry0);
    let sz: f32 = Self::s_curve(rz0);

    let mut u: f32 = self.at(b00 + bz0, rx0, ry0, rz0);
    let mut v: f32 = self.at(b10 + bz0, rx1, ry0, rz0);
    let mut a: f32 = Self::lerp(sx, u, v);

    u = self.at(b01 + bz0, rx0, ry1, rz0);
    v = self.at(b11 + bz0, rx1, ry1, rz0);

    let mut b: f32 = Self::lerp(sx, u, v);
    let c: f32 = Self::lerp(sy, a, b);

    u = self.at(b00 + bz1, rx0, ry0, rz1);
    v = self.at(b10 + bz1, rx1, ry0, rz1);
    a = Self::lerp(sx, u, v);

    u = self.at(b01 + bz1, rx0, ry1, rz1);
    v = self.at(b11 + bz1, rx1, ry1, rz1);
    b = Self::lerp(sx, u, v);

    let d: f32 = Self::lerp(sy, a, b);

    1.5 * Self::lerp(sz, c, d)
  }

  /// `PN_SETUP`: the lattice cells either side of a coordinate and the offsets into them.
  fn setup(coordinate: f32) -> (usize, usize, f32, f32) {
    let shifted: f32 = coordinate + 10_000.0;
    let truncated: i32 = shifted as i32;
    let b0: usize = (truncated & (Self::TABLE as i32 - 1)) as usize;
    let b1: usize = (b0 + 1) & (Self::TABLE - 1);
    let r0: f32 = shifted - truncated as f32;

    (b0, b1, r0, r0 - 1.0)
  }

  /// `AT`: the gradient's dot product with the offset.
  fn at(&self, index: usize, rx: f32, ry: f32, rz: f32) -> f32 {
    let gradient: &[f32; 3] = &self.gradients[index];

    rx * gradient[0] + ry * gradient[1] + rz * gradient[2]
  }

  fn s_curve(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
  }

  fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
  }
}

#[cfg(test)]
mod tests {
  use glam::Vec3;

  use super::ParticleNoise;

  #[test]
  fn fills_unit_gradients_and_repeats_the_tables_past_the_period() {
    let noise: &ParticleNoise = ParticleNoise::get();

    for index in 0..256 {
      let [x, y, z] = noise.gradients[index];

      assert!(((x * x + y * y + z * z).sqrt() - 1.0).abs() < 1e-5);
      assert_eq!(noise.gradients[256 + index], noise.gradients[index]);
      assert_eq!(noise.permutation[256 + index], noise.permutation[index]);
    }
  }

  #[test]
  fn is_zero_on_every_lattice_point() {
    let noise: &ParticleNoise = ParticleNoise::get();

    assert_eq!(noise.sample(Vec3::new(3.0, -7.0, 12.0)), 0.0);
  }

  #[test]
  fn sums_no_octaves_to_nothing() {
    assert_eq!(ParticleNoise::get().fractal_sum(Vec3::new(0.3, 0.4, 0.5), 2.0, 0), 0.0);
  }
}
