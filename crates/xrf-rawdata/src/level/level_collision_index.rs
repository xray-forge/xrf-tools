use std::collections::HashMap;

use xrf_level::{LevelCformFace, LevelCformGeometry, LevelCformTracer, LevelSurfaceMaterials};
use xrf_math::Vector3d;

/// What drew over a collision face, as the decompile finds out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LevelCollisionCover {
  /// No render triangle lies on it, so far.
  None,
  /// A static render triangle lies on it.
  Static,
  /// Only a tree's triangle lies on it: a trunk's collision, which comes back with the tree.
  Tree,
}

/// Where a render triangle lies on a level's collision form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelCollisionLookup {
  /// On a collision face of the very same three points, of this game material.
  Exact(u16),
  /// On a collision face a short ray from it meets: the level compiler split the render face and kept the collision face
  /// whole.
  Traced(u16),
  /// On none: the surface was built with a compiler shader of no collision.
  None,
}

impl LevelCollisionLookup {
  /// The game material it lies on, if any.
  pub const fn get_material(self) -> Option<u16> {
    match self {
      Self::Exact(material) | Self::Traced(material) => Some(material),
      Self::None => None,
    }
  }
}

/// A level's collision form (`level.cform`) indexed to tell which collision face each render triangle lies on, and so
/// which faces no render triangle covers: collision-only geometry, built from surfaces that collide without rendering.
pub struct LevelCollisionIndex {
  geometry: LevelCformGeometry,
  /// The faces on each set of three exact points.
  exact: HashMap<[[u32; 3]; 3], Vec<u32>>,
  tracer: LevelCformTracer,
  covers: Vec<LevelCollisionCover>,
}

impl LevelCollisionIndex {
  pub fn new(geometry: LevelCformGeometry) -> Self {
    let mut exact: HashMap<[[u32; 3]; 3], Vec<u32>> = HashMap::with_capacity(geometry.get_faces().len());

    for (face, index) in geometry.get_faces().iter().zip(0u32..) {
      exact
        .entry(Self::get_key(&geometry.get_triangle(face)))
        .or_default()
        .push(index);
    }

    Self {
      tracer: LevelCformTracer::new(&geometry),
      covers: vec![LevelCollisionCover::None; geometry.get_faces().len()],
      geometry,
      exact,
    }
  }

  /// Where a static render triangle of `points` lies, marking the collision face it lies on as covered.
  pub fn find_static(&mut self, points: &[Vector3d; 3]) -> LevelCollisionLookup {
    self.find(points, LevelCollisionCover::Static)
  }

  /// Marks the collision face a tree's triangle of `points`, placed in the level, lies on, where no static triangle
  /// covers it already.
  pub fn cover_tree(&mut self, points: &[Vector3d; 3]) {
    self.find(points, LevelCollisionCover::Tree);
  }

  /// The collision faces no render triangle lies on, with their index in the form.
  pub fn get_uncovered(&self) -> impl Iterator<Item = (u32, &LevelCformFace)> {
    self.covered(LevelCollisionCover::None)
  }

  /// How many collision faces only a tree's triangles lie on.
  pub fn count_tree_covered(&self) -> usize {
    self.covered(LevelCollisionCover::Tree).count()
  }

  /// How many faces the form holds.
  pub fn count_faces(&self) -> usize {
    self.covers.len()
  }

  /// A face's three corners.
  pub fn get_triangle(&self, face: &LevelCformFace) -> [Vector3d; 3] {
    self.geometry.get_triangle(face)
  }

  fn covered(&self, cover: LevelCollisionCover) -> impl Iterator<Item = (u32, &LevelCformFace)> {
    self
      .geometry
      .get_faces()
      .iter()
      .zip(0u32..)
      .zip(&self.covers)
      .filter(move |(_, it)| **it == cover)
      .map(|((face, index), _)| (index, face))
  }

  /// The face on the triangle's very points, else the one a ray from just above its centre meets, then one from just
  /// below, which catches a face the first misses at an edge; each found face covered by `cover` where not yet.
  fn find(&mut self, points: &[Vector3d; 3], cover: LevelCollisionCover) -> LevelCollisionLookup {
    if let Some(faces) = self.exact.get(&Self::get_key(points)) {
      for face in faces {
        Self::mark(&mut self.covers[*face as usize], cover);
      }

      return LevelCollisionLookup::Exact(self.geometry.get_faces()[faces[0] as usize].material);
    }

    let [a, b, c] = points;
    let cross: Vector3d = (b - a).cross(&(c - a));
    let length: f32 = cross.length();

    // A degenerate triangle faces nowhere, so no ray finds what it lies on.
    if !length.is_normal() {
      return LevelCollisionLookup::None;
    }

    let normal: Vector3d = &cross / length;
    let centre: Vector3d = &(&(a + b) + c) / 3.0;
    let lift: Vector3d = &normal * LevelSurfaceMaterials::RAY_LIFT;
    let range: f32 = LevelSurfaceMaterials::RAY_LIFT * 2.0;
    let found: Option<(u32, u16)> = self
      .tracer
      .get_nearest_face(&(&centre + &lift), &-&normal, range)
      .or_else(|| self.tracer.get_nearest_face(&(&centre - &lift), &normal, range));

    match found {
      Some((face, material)) => {
        Self::mark(&mut self.covers[face as usize], cover);
        LevelCollisionLookup::Traced(material)
      }
      None => LevelCollisionLookup::None,
    }
  }

  /// A static triangle covers a face for good; a tree's covers one nothing covers yet.
  fn mark(current: &mut LevelCollisionCover, cover: LevelCollisionCover) {
    if *current == LevelCollisionCover::None || cover == LevelCollisionCover::Static {
      *current = cover;
    }
  }

  /// A triangle by its three points' bits, in a fixed order, so a face matches whatever corner it starts from.
  fn get_key(points: &[Vector3d; 3]) -> [[u32; 3]; 3] {
    let mut key: [[u32; 3]; 3] = points.map(|point| [point.x.to_bits(), point.y.to_bits(), point.z.to_bits()]);

    key.sort_unstable();
    key
  }
}
