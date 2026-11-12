use std::collections::HashMap;

use glam::Vec3;
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, Tracer, Triangle};
use xrf_level::{LevelCformFace, LevelCformGeometry, LevelSurfaceMaterials};
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
  /// A collision-only triangle of a library object found placed lies on it, which comes back with the object.
  Instance,
}

/// Where a render triangle lies on a level's collision form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelCollisionLookup {
  /// On a collision face of the very same three points, of this game material.
  Exact(u16),
  /// On the collision face its normal through its centre crosses nearest it: the level compiler split the render face and
  /// kept the collision face whole, or moved its points.
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
  tracer: BvhTracer,
  covers: Vec<LevelCollisionCover>,
}

impl LevelCollisionIndex {
  /// What a render triangle lying on no collision face carries in [`Self::cover_under_static`]'s index: past every
  /// material id a face holds, fourteen bits (`LevelCformFace::MATERIAL_MASK`).
  const NO_MATERIAL: u16 = u16::MAX;

  pub fn new(geometry: LevelCformGeometry) -> Self {
    let mut exact: HashMap<[[u32; 3]; 3], Vec<u32>> = HashMap::with_capacity(geometry.get_faces().len());

    for (face, index) in geometry.get_faces().iter().zip(0u32..) {
      exact
        .entry(Self::get_key(&geometry.get_triangle(face)))
        .or_default()
        .push(index);
    }

    Self {
      tracer: geometry.build_tracer(),
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

  /// Marks the collision face a placed library object's collision-only triangle of `points` lies on, where no static
  /// triangle covers it already: it comes back with the object.
  pub fn cover_instance(&mut self, points: &[Vector3d; 3]) {
    self.find(points, LevelCollisionCover::Instance);
  }

  /// Marks each face of `sector` no render triangle's centre found that lies under one of `triangles`, the sector's
  /// static render triangles, lying itself on a face of the same game material: a face xrLC kept whole where it split
  /// the render faces over it, so their centres land on its neighbours and miss it. Each is found as a render triangle
  /// finds its face, but the other way: the render triangle its normal through its centre crosses nearest it. A face of
  /// another material under a render triangle is collision of its own, such as a collision-only prop laid on a floor,
  /// and stays uncovered.
  ///
  /// # Errors
  ///
  /// Returns an error when the triangles are too many to index, past `u32` corners.
  pub fn cover_under_static(&mut self, sector: u16, triangles: &[[Vector3d; 3]]) -> XrfResult {
    let uncovered: Vec<usize> = (0..self.covers.len())
      .filter(|index| {
        self.covers[*index] == LevelCollisionCover::None && self.geometry.get_faces()[*index].sector == sector
      })
      .collect();

    if uncovered.is_empty() || triangles.is_empty() {
      return Ok(());
    }

    u32::try_from(triangles.len())
      .map_err(|_| XrfError::new_invalid_error("A sector holds more render triangles than a tracer indexes"))?;

    // The material each render triangle lies on, `NO_MATERIAL` where it lies on none.
    let materials: Vec<u16> = triangles
      .iter()
      .map(|points| self.locate(points).1.get_material().unwrap_or(Self::NO_MATERIAL))
      .collect();
    let render: BvhTracer = BvhTracer::of_triangles(
      triangles
        .iter()
        .map(|points| Triangle::new(points.map(Vec3::from)))
        .collect(),
    );

    for index in uncovered {
      let face: &LevelCformFace = &self.geometry.get_faces()[index];
      let Some((centre, normal)) = Self::get_centre_and_normal(&self.geometry.get_triangle(face)) else {
        continue;
      };

      if render
        .get_face_nearest_along(
          Vec3::from(&centre),
          Vec3::from(&normal),
          LevelSurfaceMaterials::COLLISION_REACH,
        )
        .is_some_and(|triangle| materials[triangle as usize] == face.material)
      {
        self.covers[index] = LevelCollisionCover::Static;
      }
    }

    Ok(())
  }

  /// The collision faces no render triangle lies on, with their index in the form.
  pub fn get_uncovered(&self) -> impl Iterator<Item = (u32, &LevelCformFace)> {
    self.covered(LevelCollisionCover::None)
  }

  /// How many collision faces only a tree's triangles lie on.
  pub fn count_tree_covered(&self) -> usize {
    self.covered(LevelCollisionCover::Tree).count()
  }

  /// How many faces only a placed library object's collision-only triangles lie on.
  pub fn count_instance_covered(&self) -> usize {
    self.covered(LevelCollisionCover::Instance).count()
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

  /// Each face found by [`Self::locate`] covered by `cover` where not yet.
  fn find(&mut self, points: &[Vector3d; 3], cover: LevelCollisionCover) -> LevelCollisionLookup {
    let (faces, lookup) = self.locate(points);

    for face in faces {
      Self::mark(&mut self.covers[face as usize], cover);
    }

    lookup
  }

  /// The faces a triangle lies on, and where: those on its very points, else the one its normal through its centre
  /// crosses nearest it within a few centimetres either side, before a face stacked close above or below.
  fn locate(&self, points: &[Vector3d; 3]) -> (Vec<u32>, LevelCollisionLookup) {
    if let Some(faces) = self.exact.get(&Self::get_key(points)) {
      return (
        faces.clone(),
        LevelCollisionLookup::Exact(self.geometry.get_faces()[faces[0] as usize].material),
      );
    }

    // A degenerate triangle faces nowhere, so no line finds what it lies on.
    let Some((centre, normal)) = Self::get_centre_and_normal(points) else {
      return (Vec::new(), LevelCollisionLookup::None);
    };

    match self.tracer.get_face_nearest_along(
      Vec3::from(&centre),
      Vec3::from(&normal),
      LevelSurfaceMaterials::COLLISION_REACH,
    ) {
      Some(face) => (
        vec![face],
        LevelCollisionLookup::Traced(self.geometry.get_faces()[face as usize].material),
      ),
      None => (Vec::new(), LevelCollisionLookup::None),
    }
  }

  /// A triangle's centre and unit normal; none for a degenerate one.
  fn get_centre_and_normal([a, b, c]: &[Vector3d; 3]) -> Option<(Vector3d, Vector3d)> {
    let cross: Vector3d = (b - a).cross(&(c - a));
    let length: f32 = cross.length();

    length.is_normal().then(|| (&(&(a + b) + c) / 3.0, &cross / length))
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
