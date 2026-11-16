use xrf_math::{Vector2d, Vector3d};

use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;

/// Gives back a texture span to a level triangle whose stored base coordinates collapsed to one point.
///
/// `level.geom` stores a base coordinate as a short over 1024 and a fraction byte riding in a tangent or binormal alpha
/// (`LevelVertexLayout::rebuild_coordinate`). A surface mapped across hundreds of metres to a fraction of a texel, as a
/// flat plane running to vanilla pripyat's eastern edge is, comes back with faces whose corners all store one coordinate,
/// their fraction bytes at 255: xrLC compiled them, but refuses them as the level stores them (`Face::Verify`, "Invalid
/// face"). Each such triangle is mapped again by its corners' positions, flattened along the axis its normal leans on
/// most, to a span just over xrLC's least from the corners' lowest coordinate, so it samples the texel the level drew it
/// with. The new coordinates depend on the corners as a set, so a two-sided face's mirror maps alike and still folds
/// with it.
pub(crate) struct CollapsedMapping;

impl CollapsedMapping {
  /// The least perimeter a face's base coordinates may span, half a texel of a 4096 texture (`Face::Verify`).
  const LEAST_SPAN: f32 = 0.5 / 4096.0;

  /// The perimeter a collapsed face is given, twice xrLC's least, so single-precision rounding keeps it over that.
  const SPREAD: f64 = 1.0 / 4096.0;

  /// How many times the spread doubles where a coordinate is so large that its rounding eats the span.
  const SPREAD_DOUBLINGS: usize = 16;

  /// `triangle` with its base coordinates spread where they span under xrLC's least; as it was where they do not, or
  /// where it has no area to spread them over.
  pub(crate) fn spread(triangle: [OgfCorner; 3]) -> [OgfCorner; 3] {
    if !Self::is_collapsed(&triangle) {
      return triangle;
    }

    let Some(flat) = Self::flatten(&triangle) else {
      return triangle;
    };
    let perimeter: f64 = (0..3)
      .map(|index| Self::get_flat_distance(flat[index], flat[(index + 1) % 3]))
      .sum();

    if !perimeter.is_normal() {
      return triangle;
    }

    let coordinates: [[f64; 2]; 3] = triangle.map(|corner| corner.texture_coordinate.to_array().map(f64::from));
    let lowest = |points: &[[f64; 2]; 3]| -> [f64; 2] {
      [0, 1].map(|axis| points.iter().map(|point| point[axis]).fold(f64::INFINITY, f64::min))
    };
    let (origin, base) = (lowest(&coordinates), lowest(&flat));
    let mut spread: f64 = Self::SPREAD;

    for _ in 0..Self::SPREAD_DOUBLINGS {
      let scale: f64 = spread / perimeter;
      let mapped: [OgfCorner; 3] = [0, 1, 2].map(|index| OgfCorner {
        texture_coordinate: Vector2d::new(
          (origin[0] + (flat[index][0] - base[0]) * scale) as f32,
          (origin[1] + (flat[index][1] - base[1]) * scale) as f32,
        ),
        ..triangle[index]
      });

      if !Self::is_collapsed(&mapped) {
        return mapped;
      }

      spread *= 2.0;
    }

    triangle
  }

  /// Whether xrLC refuses `triangle`'s base coordinates: their perimeter, measured in single precision as
  /// `Face::Verify` measures it, under half a texel of a 4096 texture.
  fn is_collapsed(triangle: &[OgfCorner; 3]) -> bool {
    let distance = |first: &Vector2d, second: &Vector2d| {
      let (u, v) = (first.x - second.x, first.y - second.y);

      (u * u + v * v).sqrt()
    };
    let [a, b, c] = triangle.map(|corner| corner.texture_coordinate);
    let span: f32 = distance(&a, &b) + distance(&b, &c) + distance(&c, &a);

    span.is_nan() || span < Self::LEAST_SPAN
  }

  /// The corners' positions on the plane of the two axes the triangle's normal leans on least, or `None` for a
  /// triangle of no normal. The normal's axis is chosen by size alone, so a mirror flattens alike.
  fn flatten(triangle: &[OgfCorner; 3]) -> Option<[[f64; 2]; 3]> {
    let [a, b, c] = triangle.map(|corner| corner.position);
    let normal: Vector3d = (&b - &a).cross(&(&c - &a));
    let leaning: [f32; 3] = [normal.x.abs(), normal.y.abs(), normal.z.abs()];
    let axis: usize = (0..3).max_by(|first, second| leaning[*first].total_cmp(&leaning[*second]))?;

    if !leaning[axis].is_normal() {
      return None;
    }

    let [first, second] = match axis {
      0 => [1, 2],
      1 => [0, 2],
      _ => [0, 1],
    };

    Some(triangle.map(|corner| {
      let position: [f32; 3] = [corner.position.x, corner.position.y, corner.position.z];

      [f64::from(position[first]), f64::from(position[second])]
    }))
  }

  /// The distance between two flattened points.
  fn get_flat_distance(first: [f64; 2], second: [f64; 2]) -> f64 {
    (first[0] - second[0]).hypot(first[1] - second[1])
  }
}
