use glam::Vec3;

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::tracer::trace_hit::TraceHit;
use crate::tracer::tracer::Tracer;
use crate::triangle::triangle::Triangle;

/// Every query answered by testing every face in order: what a hierarchy must agree with.
pub(crate) struct BruteTracer {
  pub(crate) triangles: Vec<Triangle>,
}

impl BruteTracer {
  fn hits(&self, ray: &Ray, facing: Facing) -> impl Iterator<Item = TraceHit> {
    self.triangles.iter().zip(0u32..).filter_map(move |(triangle, face)| {
      triangle
        .cross(ray, facing)
        .map(|crossing| TraceHit::new(face, crossing))
    })
  }
}

impl Tracer for BruteTracer {
  fn get_face_count(&self) -> usize {
    self.triangles.len()
  }

  fn get_triangle(&self, face: u32) -> Triangle {
    self.triangles[face as usize]
  }

  fn for_each_hit(&self, ray: &Ray, visit: impl FnMut(TraceHit)) {
    self.hits(ray, Facing::Front).for_each(visit);
  }

  fn for_each_packet_hit(&self, rays: &[Ray], mut visit: impl FnMut(usize, TraceHit)) {
    for (index, ray) in rays.iter().enumerate() {
      self.for_each_hit(ray, |hit| visit(index, hit));
    }
  }

  fn for_each_face_in_box(&self, center: Vec3, half: Vec3, mut visit: impl FnMut(u32)) {
    for (triangle, face) in self.triangles.iter().zip(0u32..) {
      if triangle.overlaps_box(center, half) {
        visit(face);
      }
    }
  }

  fn get_nearest_hit(&self, ray: &Ray) -> Option<TraceHit> {
    self
      .hits(ray, Facing::Front)
      .reduce(|best, hit| if hit.distance < best.distance { hit } else { best })
  }

  fn is_blocked(&self, ray: &Ray) -> bool {
    self.hits(ray, Facing::Both).next().is_some()
  }

  fn get_face_nearest_along(&self, point: Vec3, direction: Vec3, reach: f32) -> Option<u32> {
    let mut nearest: Option<(f32, u32)> = None;

    for along in [direction, -direction] {
      let ray: Ray = Ray::new(point - along * reach, along, reach * 2.0);

      for hit in self.hits(&ray, Facing::Both) {
        let off: f32 = (hit.distance - reach).abs();

        if nearest.is_none_or(|(best, _)| off < best) {
          nearest = Some((off, hit.face));
        }
      }
    }

    nearest.map(|(_, face)| face)
  }
}

/// A deterministic sequence in `[0, 1)`, so a fixture is the same every run.
pub(crate) struct Sequence(pub(crate) u64);

impl Sequence {
  pub(crate) fn next(&mut self) -> f32 {
    self.0 = self
      .0
      .wrapping_mul(6_364_136_223_846_793_005)
      .wrapping_add(1_442_695_040_888_963_407);

    (self.0 >> 40) as f32 / (1u64 << 24) as f32
  }

  /// A point in the cube from the origin to `size` along each axis.
  pub(crate) fn point(&mut self, size: f32) -> Vec3 {
    Vec3::new(self.next(), self.next(), self.next()) * size
  }

  /// A unit direction, any way.
  pub(crate) fn direction(&mut self) -> Vec3 {
    Vec3::new(self.next() - 0.5, self.next() - 0.5, self.next() - 0.5).normalize_or(Vec3::Y)
  }
}

/// `count` small triangles strewn through a cube `size` metres across, wound every way.
pub(crate) fn soup(count: usize, size: f32, sequence: &mut Sequence) -> Vec<Triangle> {
  (0..count)
    .map(|_| {
      let at: Vec3 = sequence.point(size);

      Triangle::new([
        at + sequence.point(2.0),
        at + sequence.point(2.0),
        at + sequence.point(2.0),
      ])
    })
    .collect()
}

/// A square of floor ten metres across at the height given, its two triangles facing up.
pub(crate) fn floor(height: f32) -> [Triangle; 2] {
  let [a, b, c, d] = [(-5.0, -5.0), (5.0, -5.0), (5.0, 5.0), (-5.0, 5.0)].map(|(x, z)| Vec3::new(x, height, z));

  [Triangle::new([a, c, b]), Triangle::new([a, d, c])]
}

/// A town of axis-aligned blocks a metre across and `height` tall on a grid three metres apart, every side two
/// triangles: boxes whose faces lie on whole coordinates, for rays along the axes starting on them.
pub(crate) fn town(blocks: u32, height: f32) -> Vec<Triangle> {
  let mut triangles: Vec<Triangle> = Vec::new();

  for row in 0..blocks {
    for column in 0..blocks {
      let low: Vec3 = Vec3::new(column as f32 * 3.0, 0.0, row as f32 * 3.0);
      let high: Vec3 = low + Vec3::new(1.0, height, 1.0);
      let corner = |x: bool, y: bool, z: bool| {
        Vec3::new(
          if x { high.x } else { low.x },
          if y { high.y } else { low.y },
          if z { high.z } else { low.z },
        )
      };

      // Each side by its four corners in order around it.
      for [a, b, c, d] in [
        [
          corner(false, false, false),
          corner(false, false, true),
          corner(false, true, true),
          corner(false, true, false),
        ],
        [
          corner(true, false, false),
          corner(true, true, false),
          corner(true, true, true),
          corner(true, false, true),
        ],
        [
          corner(false, false, false),
          corner(true, false, false),
          corner(true, false, true),
          corner(false, false, true),
        ],
        [
          corner(false, true, false),
          corner(false, true, true),
          corner(true, true, true),
          corner(true, true, false),
        ],
        [
          corner(false, false, false),
          corner(false, true, false),
          corner(true, true, false),
          corner(true, false, false),
        ],
        [
          corner(false, false, true),
          corner(true, false, true),
          corner(true, true, true),
          corner(false, true, true),
        ],
      ] {
        triangles.push(Triangle::new([a, c, b]));
        triangles.push(Triangle::new([a, d, c]));
      }
    }
  }

  triangles
}

/// Every hit of a query, ordered by face so two walks compare whatever order they visit in.
pub(crate) fn sorted(mut hits: Vec<TraceHit>) -> Vec<TraceHit> {
  hits.sort_by_key(|hit| hit.face);
  hits
}

/// Every hit a tracer reports along a ray, ordered by face.
pub(crate) fn every_hit(tracer: &impl Tracer, ray: &Ray) -> Vec<TraceHit> {
  let mut hits: Vec<TraceHit> = Vec::new();

  tracer.for_each_hit(ray, |hit| hits.push(hit));

  sorted(hits)
}

/// Every face a tracer reports in a box, ascending.
pub(crate) fn faces_in_box(tracer: &impl Tracer, center: Vec3, half: Vec3) -> Vec<u32> {
  let mut faces: Vec<u32> = Vec::new();

  tracer.for_each_face_in_box(center, half, |face| faces.push(face));
  faces.sort_unstable();

  faces
}
