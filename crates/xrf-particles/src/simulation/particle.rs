use glam::{Vec3, Vec4};

/// `PAPI::Particle`: one live particle of an effect, in the space its effect was placed in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
  /// `rot.x`: the sprite's turn in radians.
  pub rotation: f32,
  pub position: Vec3,
  /// `posB`: the position before the last move, which collision and children's velocities read.
  pub previous_position: Vec3,
  pub velocity: Vec3,
  pub size: Vec3,
  /// Red, green, blue and alpha in `0..=1`, quantised to bytes wherever the engine packs them.
  pub color: Vec4,
  /// Seconds lived, starting from the source's initial age.
  pub age: f32,
  /// The frame times 255, as the engine stores it in a `u16`.
  pub frame: u16,
  /// `ANIMATE_CCW`: plays its frames backwards.
  pub is_reversed: bool,
}

impl Particle {
  /// Bytes in a packed colour channel.
  const CHANNEL: f32 = 255.0;

  /// `color_rgba_f` then `Fcolor::set`: each channel floored to a byte, clamped, and read back as a fraction.
  pub fn quantize_color(color: Vec4) -> Vec4 {
    (color * Self::CHANNEL)
      .floor()
      .clamp(Vec4::ZERO, Vec4::splat(Self::CHANNEL))
      / Self::CHANNEL
  }

  /// The frame it shows, `iFloor(frame / 255)`.
  pub fn get_frame_index(&self) -> u32 {
    (self.frame as f32 / Self::CHANNEL).floor() as u32
  }
}

#[cfg(test)]
mod tests {
  use glam::Vec4;

  use super::Particle;

  #[test]
  fn floors_a_colour_to_bytes_and_clamps_it() {
    let quantized: Vec4 = Particle::quantize_color(Vec4::new(0.5, 1.2, -0.1, 0.999));

    assert_eq!(quantized, Vec4::new(127.0 / 255.0, 1.0, 0.0, 254.0 / 255.0));
  }
}
