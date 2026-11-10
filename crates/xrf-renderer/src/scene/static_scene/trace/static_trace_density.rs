use glam::{Vec2, Vec3};

use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

/// A cut-out triangle's density word, which the scene trace's test of its holes reads in place of its corners'
/// positions (`cut_out_lod` in `shaders/common/scene_trace.wgsl`). The low half holds the base-2 logarithm of its base
/// coordinate's units a metre of it spans in its geometry's own space, fixed point; the high half the direction of its
/// corners' cross, octahedral, a byte a channel, by which an instance scaling unevenly stretches its area where it is
/// read (`StaticTraceInstance`).
pub(crate) struct StaticTraceDensity;

impl StaticTraceDensity {
  /// Steps a unit of the logarithm is stored in, and the offset that keeps it unsigned: a 256th of a level of detail,
  /// over densities from 2⁻¹²⁸ to 2¹²⁸.
  pub(crate) const STEPS: f32 = 256.0;
  pub(crate) const BIAS: f32 = 128.0;

  /// The density words of a geometry's triangles, its `indices` counted from its first vertex (`vertex_start`) in its
  /// layout's `words`. A triangle of no area is stored densest and one whose base coordinates span none sparsest, as
  /// the test reads it from its corners; a corner past the words reads as none.
  pub(crate) fn measure(layout: StaticLayout, words: &[u32], vertex_start: u32, indices: &[u32]) -> Vec<u32> {
    let corner = |index: u32| -> (Vec3, Vec2) {
      let at: usize = (vertex_start + index) as usize * StaticLayout::STRIDE as usize;

      words
        .get(at..at + StaticLayout::STRIDE as usize)
        .map_or((Vec3::ZERO, Vec2::ZERO), |vertex| {
          (
            Vec3::from_array([0, 1, 2].map(|axis| f32::from_bits(vertex[StaticLayout::POSITION + axis]))),
            Self::get_base_uv(layout, vertex),
          )
        })
    };

    indices
      .as_chunks::<3>()
      .0
      .iter()
      .map(|triangle| Self::pack(triangle.map(corner)))
      .collect()
  }

  /// A triangle's word from its three corners, each its position and base coordinate.
  fn pack([(p0, uv0), (p1, uv1), (p2, uv2)]: [(Vec3, Vec2); 3]) -> u32 {
    let across: Vec3 = (p1 - p0).cross(p2 - p0);
    let area: f32 = across.length();
    let uv_area: f32 = (uv1 - uv0).perp_dot(uv2 - uv0).abs();
    let logarithm: f32 = if uv_area == 0.0 {
      -Self::BIAS
    } else if area == 0.0 {
      Self::BIAS
    } else {
      0.5 * (uv_area / area).log2()
    };
    let stored: u32 = ((logarithm + Self::BIAS) * Self::STEPS)
      .round()
      .clamp(0.0, f32::from(u16::MAX)) as u32;
    let [x, y]: [u32; 2] = Self::encode_direction(across.try_normalize().unwrap_or(Vec3::Z))
      .to_array()
      .map(|value| ((value.clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0).round() as u32);

    stored | (x << 16) | (y << 24)
  }

  /// A vertex's base coordinate as the scene trace decodes it from its layout's words (`scene_base_uvs`): a tree's two
  /// shorts over 2048, a spawned model's two floats, a sector's two shorts with their fraction in its tangent's and
  /// binormal's fourth bytes, over 1024.
  fn get_base_uv(layout: StaticLayout, vertex: &[u32]) -> Vec2 {
    let word: u32 = vertex[StaticLayout::UV];
    let shorts: Vec2 = Vec2::new(
      f32::from((word as u16).cast_signed()),
      f32::from(((word >> 16) as u16).cast_signed()),
    );

    match layout {
      StaticLayout::Tree => shorts / 2048.0,
      StaticLayout::Model => Vec2::new(f32::from_bits(word), f32::from_bits(vertex[StaticLayout::EXTRA])),
      StaticLayout::Baked => {
        let fraction = |packed: u32| f32::from((packed >> 24) as u8) / 255.0;

        (shorts
          + Vec2::new(
            fraction(vertex[StaticLayout::TANGENT]),
            fraction(vertex[StaticLayout::BINORMAL]),
          ))
          / 1024.0
      }
    }
  }

  /// A unit direction folded onto two channels, as `octahedral_encode` in `shaders/common/octahedral.wgsl` folds it.
  fn encode_direction(direction: Vec3) -> Vec2 {
    let folded: Vec3 = direction / (direction.x.abs() + direction.y.abs() + direction.z.abs());
    let sign = |value: f32| if value >= 0.0 { 1.0 } else { -1.0 };

    if folded.z >= 0.0 {
      folded.truncate()
    } else {
      Vec2::new(
        (1.0 - folded.y.abs()) * sign(folded.x),
        (1.0 - folded.x.abs()) * sign(folded.y),
      )
    }
  }
}
