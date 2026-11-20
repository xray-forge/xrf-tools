use glam::{DVec2, DVec3};

use crate::level::instance::level_instance_fit::LevelInstanceFit;
use crate::level::instance::level_instance_match::LevelInstanceMatch;
use crate::level::instance::level_instance_triangle::LevelInstanceTriangle;
use crate::level::instance::level_instance_triangles::LevelInstanceTriangles;
use crate::level::instance::level_library_object::LevelLibraryObject;
use crate::level::instance::level_library_triangle::LevelLibraryTriangle;

/// Finds library objects placed in a level's static geometry, which the Build merged into it.
///
/// A rigid placement keeps edge lengths, so each level triangle matching an anchor (an object triangle of rare texture
/// and lengths) gives a candidate placement. A candidate stands once a sample of the object's largest triangles and
/// then enough of its area are found; it then claims the level triangles on its surface. Objects are tried largest
/// first, so one contained in a larger object is not found inside it.
///
/// A placement is found only where the level keeps one of the object's larger triangles whole to anchor on; the pieces
/// xrLC cut the others into still count towards its area.
pub(crate) struct LevelInstanceMatcher;

impl LevelInstanceMatcher {
  /// How much of an object's rendered area a placement must find among the level's triangles.
  pub(crate) const COVERAGE: f64 = 0.8;

  /// How many of an object's triangles are tried as anchors, and how many of its largest a candidate checks first.
  const ANCHORS: usize = 3;
  const ANCHOR_SCAN: usize = 64;
  const SAMPLE: usize = 12;

  /// The scales a scaled placement is looked for at, and how near one a scale is left to the rigid search.
  const SCALES: std::ops::RangeInclusive<f64> = 0.25..=4.0;
  const RIGID_SCALE: f64 = 1e-3;

  /// Every placement of `objects` in `triangles` covering at least `coverage`, claiming its triangles as found; each
  /// match names its object by its index in `objects`.
  pub(crate) fn find(
    objects: &[LevelLibraryObject],
    triangles: &mut LevelInstanceTriangles,
    coverage: f64,
  ) -> Vec<LevelInstanceMatch> {
    let mut matches: Vec<LevelInstanceMatch> = Vec::new();

    for (index, object) in objects.iter().enumerate() {
      for is_scaled in [false, true] {
        // Anchors are chosen again each round: a placement's claims may have taken every whole copy of an anchor.
        loop {
          let found: Vec<LevelInstanceMatch> = Self::search(index, object, triangles, coverage, is_scaled);

          if found.is_empty() {
            break;
          }

          matches.extend(found);
        }
      }
    }

    matches
  }

  /// One round of the search for `object`, rigid or scaled: each placement its anchors find, claimed as found.
  fn search(
    index: usize,
    object: &LevelLibraryObject,
    triangles: &mut LevelInstanceTriangles,
    coverage: f64,
    is_scaled: bool,
  ) -> Vec<LevelInstanceMatch> {
    let mut matches: Vec<LevelInstanceMatch> = Vec::new();

    for anchor in Self::get_anchors(object, triangles, is_scaled) {
      let anchor: &LevelLibraryTriangle = &object.triangles[anchor];
      let edges: [f64; 3] = LevelLibraryTriangle::get_edges(&anchor.points);
      let candidates: Vec<u32> = if is_scaled {
        triangles.find_by_ratios(anchor.texture, &edges)
      } else {
        triangles.find_by_edges(anchor.texture, &edges)
      };

      for candidate in candidates {
        // A placement found from an earlier candidate claims this one.
        if triangles.triangles[candidate as usize].is_claimed {
          continue;
        }

        let level: [DVec3; 3] = triangles.triangles[candidate as usize].points;

        for order in LevelInstanceTriangles::PERMUTATIONS {
          let to: [DVec3; 3] = order.map(|corner| level[corner]);

          if !Self::is_alike(&anchor.points, &to, is_scaled) {
            continue;
          }

          let Some(fit) = LevelInstanceFit::of_triangles(&anchor.points, &to, is_scaled) else {
            continue;
          };

          if let Some(found) = Self::try_place(index, object, fit, triangles, coverage, is_scaled) {
            for triangle in &found.claimed {
              triangles.claim(*triangle);
            }

            matches.push(found);
            break;
          }
        }
      }
    }

    matches
  }

  /// The object placed by `fit`, refitted over the matched corners, where its sample and then its area are found.
  fn try_place(
    index: usize,
    object: &LevelLibraryObject,
    fit: LevelInstanceFit,
    triangles: &LevelInstanceTriangles,
    coverage: f64,
    is_scaled: bool,
  ) -> Option<LevelInstanceMatch> {
    let sample: usize = object.triangles.len().min(Self::SAMPLE);
    // A sampled triangle counts whole, or where the level holds at least half of it in pieces.
    let sampled: usize = object.triangles[..sample]
      .iter()
      .filter(|triangle| {
        let placed: [DVec3; 3] = triangle.points.map(|point| fit.apply(point));

        triangles
          .find_at(triangle.texture, &placed, triangle.uvs.as_ref())
          .is_some()
          || Self::get_piece_area(triangle, &placed, triangles) >= 0.5 * triangle.area
      })
      .count();

    if sampled * 3 < sample * 2 {
      return None;
    }

    let (_, pairs) = Self::measure(object, &fit, triangles);
    let fit: LevelInstanceFit = LevelInstanceFit::of_pairs(&pairs, is_scaled).unwrap_or(fit);
    let (found, pairs) = Self::measure(object, &fit, triangles);

    if found < coverage * object.area {
      return None;
    }

    let error: f64 = (pairs
      .iter()
      .map(|(from, to)| fit.apply(*from).distance_squared(*to))
      .sum::<f64>()
      / pairs.len().max(1) as f64)
      .sqrt();

    let (claimed, meshes): (Vec<u32>, Vec<usize>) = Self::get_claimed(object, &fit, triangles).into_iter().unzip();

    Some(LevelInstanceMatch {
      object: index,
      claimed,
      meshes,
      fit,
      coverage: found / object.area,
      error,
    })
  }

  /// The area of `object` placed by `fit` that the level holds, whole or in pieces, and each whole triangle's corners
  /// paired with the nearest level corners.
  fn measure(
    object: &LevelLibraryObject,
    fit: &LevelInstanceFit,
    triangles: &LevelInstanceTriangles,
  ) -> (f64, Vec<(DVec3, DVec3)>) {
    let mut found: f64 = 0.0;
    let mut pairs: Vec<(DVec3, DVec3)> = Vec::new();

    for triangle in &object.triangles {
      let placed: [DVec3; 3] = triangle.points.map(|point| fit.apply(point));
      let Some(level) = triangles.find_at(triangle.texture, &placed, triangle.uvs.as_ref()) else {
        found += Self::get_piece_area(triangle, &placed, triangles).min(triangle.area);
        continue;
      };

      {
        let level: &[DVec3; 3] = &triangles.triangles[level as usize].points;

        found += triangle.area;

        for (from, at) in triangle.points.iter().zip(&placed) {
          let nearest: DVec3 = *level
            .iter()
            .min_by(|left, right| left.distance_squared(*at).total_cmp(&right.distance_squared(*at)))
            .unwrap_or(at);

          pairs.push((*from, nearest));
        }
      }
    }

    (found, pairs)
  }

  /// The level's whole copies of `triangle` at `placed` that one placement takes: one facing along, plus one facing
  /// away where two-sided. A further copy belongs to another object placed on the same spot.
  fn get_whole(triangle: &LevelLibraryTriangle, placed: &[DVec3; 3], triangles: &LevelInstanceTriangles) -> Vec<u32> {
    let normal: DVec3 = (placed[1] - placed[0]).cross(placed[2] - placed[0]);
    let found: Vec<u32> = triangles.find_all_at(triangle.texture, placed, triangle.uvs.as_ref());
    let facing = |is_along: bool| {
      found.iter().copied().find(|copy| {
        let points: &[DVec3; 3] = &triangles.triangles[*copy as usize].points;

        ((points[1] - points[0]).cross(points[2] - points[0]).dot(normal) > 0.0) == is_along
      })
    };

    match (facing(true), triangle.is_two_sided.then(|| facing(false)).flatten()) {
      (Some(along), Some(away)) => vec![along, away],
      (Some(one), None) | (None, Some(one)) => vec![one],
      (None, None) => found.first().copied().into_iter().collect(),
    }
  }

  /// The area of the level's pieces of `triangle` placed at `placed` ([`Self::get_pieces`]).
  fn get_piece_area(triangle: &LevelLibraryTriangle, placed: &[DVec3; 3], triangles: &LevelInstanceTriangles) -> f64 {
    Self::get_pieces(triangle, placed, triangles)
      .iter()
      .map(|piece| LevelLibraryTriangle::get_area(&triangles.triangles[*piece as usize].points))
      .sum()
  }

  /// The unclaimed level triangles of `triangle`'s texture lying on it at `placed` and mapped as it is (up to a shift
  /// of whole repeats): itself, its mirror, and the pieces xrLC cut it into.
  ///
  /// The mapping tells apart overlapping copies on one plane, such as GSC's rows of lamps.
  fn get_pieces(triangle: &LevelLibraryTriangle, placed: &[DVec3; 3], triangles: &LevelInstanceTriangles) -> Vec<u32> {
    let margin: DVec3 = DVec3::splat(LevelInstanceTriangles::TOLERANCE);
    let min: DVec3 = placed[0].min(placed[1]).min(placed[2]);
    let max: DVec3 = placed[0].max(placed[1]).max(placed[2]);

    triangles
      .find_within(triangle.texture, min - margin, max + margin)
      .into_iter()
      .filter(|piece| {
        let piece: &LevelInstanceTriangle = &triangles.triangles[*piece as usize];

        piece.points.iter().all(|point| Self::is_on_triangle(*point, placed))
          && triangle
            .uvs
            .is_none_or(|uvs| Self::is_mapped_alike(piece, placed, &uvs))
      })
      .collect()
  }

  /// Whether `piece`'s texture coordinates are those `uvs` give at its corners' places on the triangle `placed`, moved
  /// by one shift of whole repeats.
  fn is_mapped_alike(piece: &LevelInstanceTriangle, placed: &[DVec3; 3], uvs: &[DVec2; 3]) -> bool {
    let expected: [DVec2; 3] = piece.points.map(|point| {
      let [u, v, w] = Self::get_barycentric(point, placed);

      uvs[0] * u + uvs[1] * v + uvs[2] * w
    });
    let shift: DVec2 = piece.uvs[0] - expected[0];
    let tolerance: f64 = LevelInstanceTriangles::UV_TOLERANCE;

    (shift - shift.round()).abs().max_element() <= tolerance
      && (1..3).all(|corner| (piece.uvs[corner] - expected[corner] - shift).abs().max_element() <= tolerance)
  }

  /// The barycentric coordinates of `point` in the triangle `points`, projected onto its plane (Ericson, 3.4).
  fn get_barycentric(point: DVec3, [a, b, c]: &[DVec3; 3]) -> [f64; 3] {
    let (first, second, offset) = (*b - *a, *c - *a, point - *a);
    let (d00, d01, d11) = (first.dot(first), first.dot(second), second.dot(second));
    let (d20, d21) = (offset.dot(first), offset.dot(second));
    let denominator: f64 = d00 * d11 - d01 * d01;

    if denominator == 0.0 {
      return [1.0, 0.0, 0.0];
    }

    let v: f64 = (d11 * d20 - d01 * d21) / denominator;
    let w: f64 = (d00 * d21 - d01 * d20) / denominator;

    [1.0 - v - w, v, w]
  }

  /// The level triangles, whole or in pieces, lying on `object` placed by `fit`, sorted, each with the lowest numbered
  /// mesh it lies on.
  fn get_claimed(
    object: &LevelLibraryObject,
    fit: &LevelInstanceFit,
    triangles: &LevelInstanceTriangles,
  ) -> Vec<(u32, usize)> {
    let mut claimed: Vec<(u32, usize)> = object
      .triangles
      .iter()
      .flat_map(|triangle| {
        let placed: [DVec3; 3] = triangle.points.map(|point| fit.apply(point));
        let found: Vec<u32> = match Self::get_whole(triangle, &placed, triangles) {
          whole if whole.is_empty() => Self::get_pieces(triangle, &placed, triangles),
          whole => whole,
        };

        found.into_iter().map(|level| (level, triangle.mesh))
      })
      .collect();

    claimed.sort_unstable();
    claimed.dedup_by_key(|(level, _)| *level);
    claimed
  }

  /// The triangles of `object` tried as anchors: its largest the level holds a match for, scalene ones first since a
  /// match fixes their corners. A large anchor magnifies its corners' rounding least across the object.
  fn get_anchors(object: &LevelLibraryObject, triangles: &LevelInstanceTriangles, is_scaled: bool) -> Vec<usize> {
    let mut anchors: Vec<(bool, usize)> = object
      .triangles
      .iter()
      .take(Self::ANCHOR_SCAN)
      .enumerate()
      .filter(|(_, triangle)| {
        let edges: [f64; 3] = LevelLibraryTriangle::get_edges(&triangle.points);

        if is_scaled {
          edges[2] >= LevelInstanceTriangles::RATIO_EDGE
            && !triangles.find_by_ratios(triangle.texture, &edges).is_empty()
        } else {
          !triangles.find_by_edges(triangle.texture, &edges).is_empty()
        }
      })
      .map(|(index, triangle)| (!Self::is_scalene(&triangle.points), index))
      .collect();

    anchors.sort_unstable();

    anchors
      .into_iter()
      .take(Self::ANCHORS)
      .map(|(_, index)| index)
      .collect()
  }

  /// Whether the two triangles' edges, corner by corner, are as long within the tolerance. Where `is_scaled`, `from`
  /// is first scaled by the perimeter ratio, which must lie in [`Self::SCALES`] and not near one.
  fn is_alike(from: &[DVec3; 3], to: &[DVec3; 3], is_scaled: bool) -> bool {
    let edge = |points: &[DVec3; 3], a: usize, b: usize| points[a].distance(points[b]);
    let pairs: [(usize, usize); 3] = [(0, 1), (1, 2), (2, 0)];
    let scale: f64 = if is_scaled {
      let perimeter = |points: &[DVec3; 3]| pairs.iter().map(|(a, b)| edge(points, *a, *b)).sum::<f64>();
      let scale: f64 = perimeter(to) / perimeter(from);

      if !(Self::SCALES.contains(&scale) && (scale - 1.0).abs() > Self::RIGID_SCALE) {
        return false;
      }

      scale
    } else {
      1.0
    };

    pairs
      .iter()
      .all(|(a, b)| (edge(from, *a, *b) * scale - edge(to, *a, *b)).abs() <= 2.0 * LevelInstanceTriangles::TOLERANCE)
  }

  /// Whether no two of the triangle's edges are as long within the tolerance, so it matches another one way only.
  fn is_scalene(points: &[DVec3; 3]) -> bool {
    let edges: [f64; 3] = LevelLibraryTriangle::get_edges(points);
    let tolerance: f64 = 4.0 * LevelInstanceTriangles::TOLERANCE;

    edges[1] - edges[0] > tolerance && edges[2] - edges[1] > tolerance
  }

  /// Whether `point` lies within the tolerance of the triangle `points`.
  fn is_on_triangle(point: DVec3, points: &[DVec3; 3]) -> bool {
    Self::get_closest_point(point, points).distance_squared(point)
      <= LevelInstanceTriangles::TOLERANCE * LevelInstanceTriangles::TOLERANCE
  }

  /// The point of the triangle `points` closest to `point` (Ericson, "Real-Time Collision Detection", 5.1.5).
  fn get_closest_point(point: DVec3, [a, b, c]: &[DVec3; 3]) -> DVec3 {
    let (ab, ac, ap) = (*b - *a, *c - *a, point - *a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));

    if d1 <= 0.0 && d2 <= 0.0 {
      return *a;
    }

    let bp: DVec3 = point - *b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));

    if d3 >= 0.0 && d4 <= d3 {
      return *b;
    }

    let vc: f64 = d1 * d4 - d3 * d2;

    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
      return *a + ab * (d1 / (d1 - d3));
    }

    let cp: DVec3 = point - *c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));

    if d6 >= 0.0 && d5 <= d6 {
      return *c;
    }

    let vb: f64 = d5 * d2 - d1 * d6;

    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
      return *a + ac * (d2 / (d2 - d6));
    }

    let va: f64 = d3 * d6 - d5 * d4;

    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
      return *b + (*c - *b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }

    let denominator: f64 = 1.0 / (va + vb + vc);

    *a + ab * (vb * denominator) + ac * (vc * denominator)
  }
}
