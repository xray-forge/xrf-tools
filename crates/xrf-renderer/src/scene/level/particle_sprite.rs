use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_math::EPS_S;
use xrf_particles::{Particle, ParticleEffectFlags, ParticleEffectInstance};

use crate::pass::particle_vertex::ParticleVertex;

/// `CParticleEffect::ParticleRenderStream`: an effect's particles as quads, each filled as `FillSprite` fills it in
/// engine space, facing the camera or aligned to its path, then mirrored into renderer space.
pub struct ParticleSprite {
  /// `Device.vCameraTop`, `vCameraRight` and `vCameraDirection`, in engine space.
  top: Vec3,
  right: Vec3,
  direction: Vec3,
}

/// `m_APDefaultRotation`'s default: the angles a still particle aligned to its path faces by.
const DEFAULT_ALIGNMENT: Vec3 = Vec3::new(-FRAC_PI_2, 0.0, 0.0);

/// What `FVF::LIT`'s colour stores a channel as.
const CHANNEL: f32 = 255.0;

impl ParticleSprite {
  /// The camera's screen axes and facing, out of its view in renderer space.
  pub fn new(view: Mat4) -> Self {
    let camera: Mat4 = view.inverse();

    Self {
      top: Self::mirror(camera.y_axis.truncate()),
      right: Self::mirror(camera.x_axis.truncate()),
      direction: Self::mirror(-camera.z_axis.truncate()),
    }
  }

  /// Appends an effect's quads, four corners a particle, each corner naming the surface it draws with.
  pub fn push(&self, effect: &ParticleEffectInstance, surface: u32, vertices: &mut Vec<ParticleVertex>) {
    let definition = effect.get_definition();
    let flags: ParticleEffectFlags = effect.get_flags();
    let alignment: Vec3 = definition
      .rotation
      .as_ref()
      .map_or(DEFAULT_ALIGNMENT, |it| Vec3::new(it.x, it.y, it.z));
    let velocity_scale: Vec3 = definition
      .velocity_scale
      .as_ref()
      .map_or(Vec3::ZERO, |it| Vec3::new(it.x, it.y, it.z));

    for m in effect.get_pool().get_particles() {
      let (lt, rb) = match &definition.frame {
        Some(frame) if flags.is(ParticleEffectFlags::FRAMED) => {
          let index: u32 = m.get_frame_index();
          let dimension: u32 = frame.frame_dimension_x.max(1);
          let size: Vec2 = Vec2::new(frame.texture_size.0, frame.texture_size.1);
          let lt: Vec2 = Vec2::new((index % dimension) as f32, (index / dimension) as f32) * size;

          (lt, lt + size)
        }
        _ => (Vec2::ZERO, Vec2::ONE),
      };
      let mut radius: Vec2 = Vec2::new(m.size.x, m.size.y) * 0.5;
      let speed: f32 = m.velocity.length();

      if flags.is(ParticleEffectFlags::VELOCITY_SCALE) {
        radius += Vec2::new(velocity_scale.x, velocity_scale.y) * speed;
      }

      let (top, right): (Vec3, Vec3) = if flags.is(ParticleEffectFlags::ALIGN_TO_PATH) {
        if speed < EPS_S && flags.is(ParticleEffectFlags::WORLD_ALIGN) {
          let basis: [Vec3; 3] = Self::set_xyz(alignment);

          (basis[2], basis[0])
        } else if speed >= EPS_S && flags.is(ParticleEffectFlags::FACE_ALIGN) {
          let k: Vec3 = m.velocity / speed;
          let up: Vec3 = if Vec3::Y.dot(k).abs() > 0.99 { Vec3::Z } else { Vec3::Y };
          let i: Vec3 = up.cross(k).normalize();

          (k.cross(i).normalize(), i)
        } else {
          let direction: Vec3 = if speed >= EPS_S {
            m.velocity / speed
          } else {
            Self::from_heading_pitch(-alignment.y, -alignment.x)
          };

          (direction, Self::normalize_safe(direction.cross(self.direction)))
        }
      } else {
        (self.top, self.right)
      };

      Self::fill(vertices, m, top, right, (lt, rb), radius, surface);
    }
  }

  /// `FillSprite`: the quad's corners about the particle along two axes, turned by its rotation, in `QuadIB`'s order.
  fn fill(
    vertices: &mut Vec<ParticleVertex>,
    m: &Particle,
    top: Vec3,
    right: Vec3,
    (lt, rb): (Vec2, Vec2),
    radius: Vec2,
    surface: u32,
  ) {
    let (sin, cos) = m.rotation.sin_cos();
    let along_right: Vec3 = (top * sin + right * cos) * radius.x;
    let along_top: Vec3 = (top * cos - right * sin) * radius.y;
    let a: Vec3 = along_top - along_right;
    let b: Vec3 = along_top + along_right;
    let color: u32 = Self::pack_color(m);
    let corner = |offset: Vec3, uv: Vec2| ParticleVertex {
      position: Self::mirror(m.position + offset),
      color,
      uv,
      surface,
      _pad: 0,
    };

    vertices.push(corner(-b, Vec2::new(lt.x, rb.y)));
    vertices.push(corner(a, lt));
    vertices.push(corner(-a, rb));
    vertices.push(corner(b, Vec2::new(rb.x, lt.y)));
  }

  /// The colour as `unpack4x8unorm` reads it, each channel to the nearest byte.
  fn pack_color(m: &Particle) -> u32 {
    let bytes: [u8; 4] = (m.color.clamp(Vec4::ZERO, Vec4::ONE) * CHANNEL)
      .round()
      .to_array()
      .map(|channel| channel as u8);

    u32::from_le_bytes(bytes)
  }

  /// `Fmatrix::setXYZ`, `setHPB(y, x, z)`: its `i`, `j` and `k` axes.
  fn set_xyz(angles: Vec3) -> [Vec3; 3] {
    let (sh, ch) = angles.y.sin_cos();
    let (sp, cp) = angles.x.sin_cos();
    let (sb, cb) = angles.z.sin_cos();
    let (cc, cs, sc, ss) = (ch * cb, ch * sb, sh * cb, sh * sb);

    [
      Vec3::new(cc - sp * ss, -cp * sb, sp * cs + sc),
      Vec3::new(sp * sc + cs, cp * cb, ss - sp * cc),
      Vec3::new(-cp * sh, sp, cp * ch),
    ]
  }

  /// `Fvector::setHP`.
  fn from_heading_pitch(heading: f32, pitch: f32) -> Vec3 {
    let (sh, ch) = heading.sin_cos();
    let (sp, cp) = pitch.sin_cos();

    Vec3::new(-cp * sh, sp, cp * ch)
  }

  /// `Fvector::normalize_safe`.
  fn normalize_safe(vector: Vec3) -> Vec3 {
    let length_sqr: f32 = vector.length_squared();

    if length_sqr > f32::MIN_POSITIVE {
      vector * (1.0 / length_sqr).sqrt()
    } else {
      vector
    }
  }

  /// Between engine and renderer space, either way: `z` the other way.
  pub fn mirror(vector: Vec3) -> Vec3 {
    Vec3::new(vector.x, vector.y, -vector.z)
  }
}

#[cfg(test)]
mod tests {
  use glam::Vec3;

  use super::ParticleSprite;

  #[test]
  fn takes_the_cameras_axes_into_engine_space() {
    // Looking down renderer `-z` is looking down engine `+z`.
    let sprite: ParticleSprite = ParticleSprite::new(glam::camera::rh::view::look_at_mat4(
      Vec3::ZERO,
      Vec3::new(0.0, 0.0, -1.0),
      Vec3::Y,
    ));

    assert!((sprite.direction - Vec3::Z).length() < 1e-6);
    assert!((sprite.top - Vec3::Y).length() < 1e-6);
    assert!((sprite.right - Vec3::X).length() < 1e-6);
  }

  #[test]
  fn stands_set_xyz_of_nothing_as_the_identity() {
    assert_eq!(ParticleSprite::set_xyz(Vec3::ZERO), [Vec3::X, Vec3::Y, Vec3::Z]);
  }
}
