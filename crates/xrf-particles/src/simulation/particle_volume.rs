use std::f32::consts::PI;

use glam::{Mat4, Vec3, Vec4Swizzles};
use xrf_math::Vector3d;

use crate::data::particle_domain::ParticleDomain;
use crate::simulation::engine_vector::EngineVector;
use crate::simulation::particle_random::ParticleRandom;

/// `pDomain`: a region particles are generated in or tested against, its fields as the file stores them.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleVolume {
  pub kind: u32,
  pub p1: Vec3,
  pub p2: Vec3,
  pub u: Vec3,
  pub v: Vec3,
  pub radius1: f32,
  pub radius2: f32,
  pub radius1_sqr: f32,
  pub radius2_sqr: f32,
}

impl ParticleVolume {
  pub const POINT: u32 = 0;
  pub const LINE: u32 = 1;
  pub const TRIANGLE: u32 = 2;
  pub const PLANE: u32 = 3;
  pub const BOX: u32 = 4;
  pub const SPHERE: u32 = 5;
  pub const CYLINDER: u32 = 6;
  pub const CONE: u32 = 7;
  pub const BLOB: u32 = 8;
  pub const DISC: u32 = 9;
  pub const RECTANGLE: u32 = 10;

  /// `Within`: whether a point is inside. Kinds with no inside answer false, and a blob answers by chance.
  pub fn is_within(&self, position: Vec3, random: &mut ParticleRandom) -> bool {
    match self.kind {
      Self::BOX => {
        !(position.x < self.p1.x
          || position.x > self.p2.x
          || position.y < self.p1.y
          || position.y > self.p2.y
          || position.z < self.p1.z
          || position.z > self.p2.z)
      }
      Self::PLANE => position.dot(self.p2) >= -self.radius1,
      Self::SPHERE => {
        let distance_sqr: f32 = (position - self.p1).length_squared();

        distance_sqr <= self.radius1_sqr && distance_sqr >= self.radius2_sqr
      }
      Self::CYLINDER | Self::CONE => {
        let offset: Vec3 = position - self.p1;
        let along: f32 = self.p2.dot(offset) * self.radius2_sqr;

        if !(0.0..=1.0).contains(&along) {
          return false;
        }

        let radial_sqr: f32 = (offset - self.p2 * along).length_squared();

        if self.kind == Self::CONE {
          radial_sqr <= (along * self.radius1) * (along * self.radius1)
            && radial_sqr >= (along * self.radius2) * (along * self.radius2)
        } else {
          radial_sqr <= self.radius1_sqr && radial_sqr >= self.radius2 * self.radius2
        }
      }
      Self::BLOB => {
        let offset: Vec3 = position - self.p1;
        let density: f32 = (offset.length_squared() * self.radius2_sqr).exp() * self.radius2;

        random.next_fraction() < density
      }
      _ => false,
    }
  }

  /// `Generate`: a random point inside.
  pub fn generate(&self, random: &mut ParticleRandom) -> Vec3 {
    match self.kind {
      Self::POINT => self.p1,
      Self::LINE => self.p1 + self.p2 * random.next_fraction(),
      Self::BOX => {
        let x: f32 = self.p1.x + (self.p2.x - self.p1.x) * random.next_fraction();
        let y: f32 = self.p1.y + (self.p2.y - self.p1.y) * random.next_fraction();
        let z: f32 = self.p1.z + (self.p2.z - self.p1.z) * random.next_fraction();

        Vec3::new(x, y, z)
      }
      Self::TRIANGLE => {
        let r1: f32 = random.next_fraction();
        let r2: f32 = random.next_fraction();

        if r1 + r2 < 1.0 {
          self.p1 + self.u * r1 + self.v * r2
        } else {
          self.p1 + self.u * (1.0 - r1) + self.v * (1.0 - r2)
        }
      }
      Self::RECTANGLE => {
        let along_u: f32 = random.next_fraction();

        self.p1 + self.u * along_u + self.v * random.next_fraction()
      }
      Self::PLANE => self.p1,
      Self::SPHERE => {
        let x: f32 = random.next_fraction();
        let y: f32 = random.next_fraction();
        let z: f32 = random.next_fraction();
        let direction: Vec3 = (Vec3::new(x, y, z) - Vec3::splat(0.5)).normalize_safe();

        if self.radius1 == self.radius2 {
          self.p1 + direction * self.radius1
        } else {
          self.p1 + direction * (self.radius2 + random.next_fraction() * (self.radius1 - self.radius2))
        }
      }
      Self::CYLINDER | Self::CONE => {
        let along: f32 = random.next_fraction();
        let theta: f32 = random.next_fraction() * 2.0 * PI;
        let radius: f32 = self.radius2 + random.next_fraction() * (self.radius1 - self.radius2);
        let mut x: f32 = radius * theta.cos();
        let mut y: f32 = radius * theta.sin();

        if self.kind == Self::CONE {
          x *= along;
          y *= along;
        }

        self.p1 + self.p2 * along + self.u * x + self.v * y
      }
      Self::BLOB => {
        let x: f32 = self.p1.x + random.next_normal(self.radius1);
        let y: f32 = self.p1.y + random.next_normal(self.radius1);
        let z: f32 = self.p1.z + random.next_normal(self.radius1);

        Vec3::new(x, y, z)
      }
      Self::DISC => {
        let theta: f32 = random.next_fraction() * 2.0 * PI;
        let radius: f32 = self.radius2 + random.next_fraction() * (self.radius1 - self.radius2);

        self.p1 + self.u * (radius * theta.cos()) + self.v * (radius * theta.sin())
      }
      _ => Vec3::ZERO,
    }
  }

  /// `s1` and `s2`: the rows reading a point's `u` and `v` coordinates in a rectangle's or triangle's plane.
  pub fn get_plane_inverse_basis(&self) -> (Vec3, Vec3) {
    let (u, v) = (self.u, self.v);
    let wx: f32 = u.y * v.z - u.z * v.y;
    let wy: f32 = u.z * v.x - u.x * v.z;
    let wz: f32 = u.x * v.y - u.y * v.x;
    let det: f32 =
      1.0 / (wz * u.x * v.y - wz * u.y * v.x - u.z * wx * v.y - u.x * v.z * wy + v.z * wx * u.y + u.z * v.x * wy);
    let s1: Vec3 = Vec3::new(v.y * wz - v.z * wy, v.z * wx - v.x * wz, v.x * wy - v.y * wx) * det;
    let s2: Vec3 = Vec3::new(u.y * wz - u.z * wy, u.z * wx - u.x * wz, u.x * wy - u.y * wx) * -det;

    (s1, s2)
  }

  /// `transform`: the source volume placed by a matrix; a box takes its turned corners' bounds (`Fbox::xform`).
  pub fn transform_from(&mut self, source: &Self, matrix: &Mat4) {
    match self.kind {
      Self::BOX => {
        let extent: Vec3 = source.p2 - source.p1;
        let edges: [Vec3; 3] = [
          matrix.x_axis.xyz() * extent.x,
          matrix.y_axis.xyz() * extent.y,
          matrix.z_axis.xyz() * extent.z,
        ];
        let mut min: Vec3 = matrix.transform_point3(source.p1);
        let mut max: Vec3 = min;

        for edge in edges {
          for axis in 0..3 {
            if edge[axis] < 0.0 {
              min[axis] += edge[axis];
            } else {
              max[axis] += edge[axis];
            }
          }
        }

        self.p1 = min;
        self.p2 = max;
      }
      Self::PLANE => {
        self.p1 = matrix.transform_point3(source.p1);
        self.p2 = matrix.transform_vector3(source.p2);
        self.radius1 = -self.p1.dot(self.p2);
      }
      Self::SPHERE | Self::BLOB | Self::POINT => {
        self.p1 = matrix.transform_point3(source.p1);
      }
      Self::LINE => {
        self.p1 = matrix.transform_point3(source.p1);
        self.p2 = matrix.transform_vector3(source.p2);
      }
      Self::CYLINDER | Self::CONE | Self::RECTANGLE | Self::TRIANGLE | Self::DISC => {
        self.p1 = matrix.transform_point3(source.p1);
        self.p2 = matrix.transform_vector3(source.p2);
        self.u = matrix.transform_vector3(source.u);
        self.v = matrix.transform_vector3(source.v);
      }
      _ => {}
    }
  }

  /// `transform_dir`: as `transform_from` with the matrix's translation dropped, for a volume of directions.
  pub fn transform_direction_from(&mut self, source: &Self, matrix: &Mat4) {
    let mut turning: Mat4 = *matrix;

    turning.w_axis = glam::Vec4::W;

    self.transform_from(source, &turning);
  }
}

impl From<&ParticleDomain> for ParticleVolume {
  fn from(domain: &ParticleDomain) -> Self {
    let to_vec3 = |vector: &Vector3d| Vec3::new(vector.x, vector.y, vector.z);

    Self {
      kind: domain.domain_type,
      p1: to_vec3(&domain.coordinates.0),
      p2: to_vec3(&domain.coordinates.1),
      u: to_vec3(&domain.basis.0),
      v: to_vec3(&domain.basis.1),
      radius1: domain.radius1,
      radius2: domain.radius2,
      radius1_sqr: domain.radius1_sqr,
      radius2_sqr: domain.radius2_sqr,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Mat4, Vec3};

  use super::ParticleVolume;
  use crate::simulation::particle_random::ParticleRandom;

  fn volume(kind: u32) -> ParticleVolume {
    ParticleVolume {
      kind,
      p1: Vec3::ZERO,
      p2: Vec3::ZERO,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    }
  }

  #[test]
  fn generates_a_box_point_from_three_draws() {
    let mut random: ParticleRandom = ParticleRandom::new(1);
    let mut cube: ParticleVolume = volume(ParticleVolume::BOX);

    cube.p1 = Vec3::new(-1.0, 0.0, 2.0);
    cube.p2 = Vec3::new(1.0, 4.0, 3.0);

    // Draws 41, 18467 and 6334 out of 32767.
    let point: Vec3 = cube.generate(&mut random);

    assert_eq!(point.x, -1.0 + 2.0 * (41.0 / 32_767.0));
    assert_eq!(point.y, 4.0 * (18_467.0 / 32_767.0));
    assert_eq!(point.z, 2.0 + 6_334.0 / 32_767.0);
  }

  #[test]
  fn tests_a_hollow_sphere_by_both_radii() {
    let mut random: ParticleRandom = ParticleRandom::new(1);
    let mut shell: ParticleVolume = volume(ParticleVolume::SPHERE);

    shell.radius1_sqr = 4.0;
    shell.radius2_sqr = 1.0;

    assert!(shell.is_within(Vec3::new(1.5, 0.0, 0.0), &mut random));
    assert!(!shell.is_within(Vec3::new(0.5, 0.0, 0.0), &mut random));
    assert!(!shell.is_within(Vec3::new(2.5, 0.0, 0.0), &mut random));
  }

  #[test]
  fn takes_the_bounds_of_a_turned_box() {
    let mut cube: ParticleVolume = volume(ParticleVolume::BOX);

    cube.p1 = Vec3::new(0.0, 0.0, 0.0);
    cube.p2 = Vec3::new(2.0, 1.0, 1.0);

    let source: ParticleVolume = cube.clone();
    let quarter_turn: Mat4 = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);

    cube.transform_from(
      &source,
      &(Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)) * quarter_turn),
    );

    assert!((cube.p1 - Vec3::new(10.0, 0.0, -2.0)).length() < 1e-5);
    assert!((cube.p2 - Vec3::new(11.0, 1.0, 0.0)).length() < 1e-5);
  }

  #[test]
  fn moves_only_a_point_of_a_sphere() {
    let mut ball: ParticleVolume = volume(ParticleVolume::SPHERE);

    ball.radius1 = 3.0;

    let source: ParticleVolume = ball.clone();

    ball.transform_direction_from(&source, &Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)));

    assert_eq!(ball.p1, Vec3::ZERO);
    assert_eq!(ball.radius1, 3.0);
  }
}
