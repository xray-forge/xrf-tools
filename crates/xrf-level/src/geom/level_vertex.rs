use serde::{Deserialize, Serialize};
use xrf_math::Vector3d;

/// One vertex of a level's render geometry, decoded from whichever declaration stored it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  /// The authored tangent, absent for a declaration that carries none.
  pub tangent: Option<Vector3d>,
  /// The authored binormal, mirrored with the tangent.
  pub binormal: Option<Vector3d>,
  /// The base texture coordinate, reassembled from its 16-bit part and the low byte the tangent and binormal carry.
  pub texture_coordinate: (f32, f32),
  /// The lightmap coordinate, for a surface xrLC lit from a lightmap.
  pub lightmap_coordinate: Option<(f32, f32)>,
  /// The baked vertex colour as `(r, g, b)`, for a surface xrLC lit from vertex colour instead.
  pub color: Option<(u8, u8, u8)>,
  /// The vertex's hemisphere term, which the normal's alpha byte carries rather than padding.
  pub hemi: u8,
}

impl LevelVertex {
  /// Decodes one `D3DCOLOR`-packed direction and the byte riding along with it.
  pub fn decode_direction(bytes: [u8; 4]) -> (Vector3d, u8) {
    (
      Vector3d {
        x: f32::from(bytes[2]) / 127.5 - 1.0,
        y: f32::from(bytes[1]) / 127.5 - 1.0,
        z: f32::from(bytes[0]) / 127.5 - 1.0,
      },
      bytes[3],
    )
  }

  /// The colour of a `D3DCOLOR` element, as the red, green and blue the packing puts on disk.
  pub const fn decode_color(bytes: [u8; 4]) -> (u8, u8, u8) {
    (bytes[2], bytes[1], bytes[0])
  }
}

#[cfg(test)]
mod tests {
  use xrf_math::Vector3d;

  use crate::geom::level_vertex::LevelVertex;

  #[test]
  fn test_direction_decodes_the_bytes_in_the_order_they_are_written() {
    // Blue, green, red, alpha: x comes from the third byte.
    let (direction, alpha): (Vector3d, u8) = LevelVertex::decode_direction([0, 128, 255, 42]);

    assert!((direction.x - 1.0).abs() < 0.01, "x is the red byte");
    assert!(direction.y.abs() < 0.01, "y is the green byte");
    assert!((direction.z + 1.0).abs() < 0.01, "z is the blue byte");
    assert_eq!(alpha, 42);
  }

  #[test]
  fn test_direction_round_trips_the_encoding_the_compiler_uses() {
    for component in [-1.0_f32, -0.5, 0.0, 0.5, 1.0] {
      let encoded: u8 = ((component + 1.0) * 127.5) as u8;
      let (direction, _): (Vector3d, u8) = LevelVertex::decode_direction([0, 0, encoded, 0]);

      assert!(
        (direction.x - component).abs() < 0.01,
        "component {component} decoded as {}",
        direction.x
      );
    }
  }

  #[test]
  fn test_color_takes_the_same_byte_order_as_a_direction() {
    assert_eq!(LevelVertex::decode_color([1, 2, 3, 4]), (3, 2, 1));
  }
}
