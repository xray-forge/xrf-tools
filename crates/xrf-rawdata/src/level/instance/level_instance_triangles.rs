use std::collections::HashMap;

use glam::{DVec2, DVec3};

use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::instance::level_library_object::{LevelLibraryObject, LevelLibraryTriangle};
use crate::level::level_visual_geometry::LevelVisualGeometry;

/// A level's static triangles of textures a library object draws, indexed four ways for instance recovery: by their
/// texture and edge lengths, which a library triangle placed rigidly keeps; by their edges' ratios, which a scaled
/// placement keeps; by the cells their corners stand in, to
/// find the triangle at three placed corners; and by the coarse cells their centres stand in, to find what a placement
/// covers. Each knows its sector's visual and triangle, and whether a placement has claimed it.
pub(crate) struct LevelInstanceTriangles {
  pub(crate) triangles: Vec<LevelInstanceTriangle>,
  by_edges: HashMap<(u32, [i64; 3]), Vec<u32>>,
  by_ratios: HashMap<(u32, [i64; 2]), Vec<u32>>,
  by_corner: HashMap<(u32, [i64; 3]), Vec<u32>>,
  by_area: HashMap<(u32, [i64; 3]), Vec<u32>>,
}

/// One static triangle of the level: its texture in the library's table, corners, and where it is read from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelInstanceTriangle {
  pub(crate) texture: u32,
  pub(crate) points: [DVec3; 3],
  pub(crate) uvs: [DVec2; 3],
  /// Its visual, by its place in the list the index was built from, and its place among the visual's triangles.
  pub(crate) visual: u32,
  pub(crate) index: u32,
  pub(crate) is_claimed: bool,
}

impl LevelInstanceTriangles {
  /// How far apart, in metres, a placed library corner and the level's may stand and be the same point: xrLC welds and
  /// moves level vertices by up to its weld distance, and stores them in single precision.
  pub(crate) const TOLERANCE: f64 = 0.005;

  /// Cells: edge lengths in steps of two tolerances, so a length and its match are at most one step apart; corners in
  /// cells of two tolerances, so a match is in the corner's cell or a neighbour; centres in two-metre cells.
  const EDGE_STEP: f64 = 2.0 * Self::TOLERANCE;
  const CORNER_CELL: f64 = 2.0 * Self::TOLERANCE;
  const AREA_CELL: f64 = 2.0;

  /// Edge ratios, the two shorter edges over the longest, in steps of a hundredth; a triangle whose longest edge is
  /// shorter than a few tolerances has ratios its rounding decides, and is not indexed by them.
  const RATIO_STEP: f64 = 0.01;
  pub(crate) const RATIO_EDGE: f64 = 8.0 * Self::TOLERANCE;

  /// The index over `visuals`' triangles whose texture `library` numbers.
  pub(crate) fn new(library: &LevelInstanceLibrary, visuals: &[&LevelVisualGeometry]) -> Self {
    let mut index: Self = Self {
      triangles: Vec::new(),
      by_edges: HashMap::new(),
      by_ratios: HashMap::new(),
      by_corner: HashMap::new(),
      by_area: HashMap::new(),
    };

    for (visual, geometry) in (0u32..).zip(visuals) {
      let Some(texture) = library.get_texture(&geometry.texture) else {
        continue;
      };

      for (triangle, corners) in (0u32..).zip(geometry.indices.as_chunks::<3>().0) {
        let corner = |corner: u16| -> Option<(DVec3, DVec2)> {
          let vertex = geometry.vertices.get(usize::from(corner))?;
          let (position, uv) = (vertex.position, vertex.texture_coordinate);

          Some((
            DVec3::new(f64::from(position.x), f64::from(position.y), f64::from(position.z)),
            DVec2::new(f64::from(uv.x), f64::from(uv.y)),
          ))
        };
        let (Some(a), Some(b), Some(c)) = (corner(corners[0]), corner(corners[1]), corner(corners[2])) else {
          continue;
        };

        index.add(LevelInstanceTriangle {
          texture,
          points: [a.0, b.0, c.0],
          uvs: [a.1, b.1, c.1],
          visual,
          index: triangle,
          is_claimed: false,
        });
      }
    }

    index
  }

  /// The index over `object`'s triangles, in its own space: a decompiled model to look for library objects in, its
  /// triangles by their place in its list.
  pub(crate) fn of_object(object: &LevelLibraryObject) -> Self {
    let mut index: Self = Self {
      triangles: Vec::new(),
      by_edges: HashMap::new(),
      by_ratios: HashMap::new(),
      by_corner: HashMap::new(),
      by_area: HashMap::new(),
    };

    for (triangle, read) in (0u32..).zip(&object.triangles) {
      index.add(LevelInstanceTriangle {
        texture: read.texture,
        points: read.points,
        uvs: read.uvs.unwrap_or([DVec2::ZERO; 3]),
        visual: 0,
        index: triangle,
        is_claimed: false,
      });
    }

    index
  }

  /// The unclaimed triangles of `texture` whose edge lengths are within a step of `edges`, shortest first.
  pub(crate) fn find_by_edges(&self, texture: u32, edges: &[f64; 3]) -> Vec<u32> {
    let key: [i64; 3] = edges.map(|edge| Self::get_cell(edge, Self::EDGE_STEP));
    let mut found: Vec<u32> = Vec::new();

    for offset in Self::NEIGHBOURS {
      let cell: [i64; 3] = [key[0] + offset[0], key[1] + offset[1], key[2] + offset[2]];

      if let Some(triangles) = self.by_edges.get(&(texture, cell)) {
        found.extend(triangles.iter().filter(|it| !self.triangles[**it as usize].is_claimed));
      }
    }

    found.sort_unstable();
    found.dedup();
    found
  }

  /// The unclaimed triangles of `texture` whose edge ratios are within a step of those of `edges`, shortest first.
  pub(crate) fn find_by_ratios(&self, texture: u32, edges: &[f64; 3]) -> Vec<u32> {
    let key: [i64; 2] = Self::get_ratios(edges);
    let mut found: Vec<u32> = Vec::new();

    for x in -1..=1 {
      for y in -1..=1 {
        if let Some(triangles) = self.by_ratios.get(&(texture, [key[0] + x, key[1] + y])) {
          found.extend(triangles.iter().filter(|it| !self.triangles[**it as usize].is_claimed));
        }
      }
    }

    found.sort_unstable();
    found.dedup();
    found
  }

  /// The unclaimed triangle of `texture` whose corners are `points`, in any order, each within the tolerance, and where
  /// `uvs` are given, whose texture coordinates are theirs moved by a whole number of repeats.
  pub(crate) fn find_at(&self, texture: u32, points: &[DVec3; 3], uvs: Option<&[DVec2; 3]>) -> Option<u32> {
    self.find_all_at(texture, points, uvs).first().copied()
  }

  /// Every unclaimed triangle [`Self::find_at`] would take: a triangle, its mirror, and copies of either that two
  /// objects placed on one spot left.
  pub(crate) fn find_all_at(&self, texture: u32, points: &[DVec3; 3], uvs: Option<&[DVec2; 3]>) -> Vec<u32> {
    let key: [i64; 3] = points[0]
      .to_array()
      .map(|value| Self::get_cell(value, Self::CORNER_CELL));
    let mut found: Vec<u32> = Vec::new();

    for offset in Self::NEIGHBOURS {
      let cell: [i64; 3] = [key[0] + offset[0], key[1] + offset[1], key[2] + offset[2]];

      found.extend(
        self
          .by_corner
          .get(&(texture, cell))
          .into_iter()
          .flatten()
          .copied()
          .filter(|candidate| {
            let triangle: &LevelInstanceTriangle = &self.triangles[*candidate as usize];

            !triangle.is_claimed
              && Self::PERMUTATIONS.iter().any(|order| {
                Self::is_same_order(&triangle.points, points, order)
                  && uvs.is_none_or(|uvs| Self::is_same_mapping(&triangle.uvs, uvs, order))
              })
          }),
      );
    }

    found.sort_unstable();
    found.dedup();
    found
  }

  /// The unclaimed triangles of `texture` whose centres stand within the box from `min` to `max`.
  pub(crate) fn find_within(&self, texture: u32, min: DVec3, max: DVec3) -> Vec<u32> {
    let low: [i64; 3] = min.to_array().map(|value| Self::get_cell(value, Self::AREA_CELL));
    let high: [i64; 3] = max.to_array().map(|value| Self::get_cell(value, Self::AREA_CELL));
    let mut found: Vec<u32> = Vec::new();

    for x in low[0]..=high[0] {
      for y in low[1]..=high[1] {
        for z in low[2]..=high[2] {
          if let Some(triangles) = self.by_area.get(&(texture, [x, y, z])) {
            found.extend(triangles.iter().filter(|it| !self.triangles[**it as usize].is_claimed));
          }
        }
      }
    }

    found
  }

  pub(crate) fn claim(&mut self, triangle: u32) {
    self.triangles[triangle as usize].is_claimed = true;
  }

  /// Every cell offset to a neighbour and the cell itself.
  const NEIGHBOURS: [[i64; 3]; 27] = {
    let mut offsets: [[i64; 3]; 27] = [[0; 3]; 27];
    let mut next: usize = 0;
    let mut x: i64 = -1;

    while x <= 1 {
      let mut y: i64 = -1;

      while y <= 1 {
        let mut z: i64 = -1;

        while z <= 1 {
          offsets[next] = [x, y, z];
          next += 1;
          z += 1;
        }

        y += 1;
      }

      x += 1;
    }

    offsets
  };

  fn add(&mut self, triangle: LevelInstanceTriangle) {
    let id: u32 = u32::try_from(self.triangles.len()).unwrap_or(u32::MAX);
    let texture: u32 = triangle.texture;
    let lengths: [f64; 3] = LevelLibraryTriangle::get_edges(&triangle.points);
    let edges: [i64; 3] = lengths.map(|edge| Self::get_cell(edge, Self::EDGE_STEP));
    let centre: DVec3 = (triangle.points[0] + triangle.points[1] + triangle.points[2]) / 3.0;

    self.by_edges.entry((texture, edges)).or_default().push(id);

    if lengths[2] >= Self::RATIO_EDGE {
      self
        .by_ratios
        .entry((texture, Self::get_ratios(&lengths)))
        .or_default()
        .push(id);
    }

    for point in &triangle.points {
      let cell: [i64; 3] = point.to_array().map(|value| Self::get_cell(value, Self::CORNER_CELL));
      let triangles: &mut Vec<u32> = self.by_corner.entry((texture, cell)).or_default();

      // A triangle with two corners in one cell is listed there once.
      if triangles.last() != Some(&id) {
        triangles.push(id);
      }
    }

    self
      .by_area
      .entry((
        texture,
        centre.to_array().map(|value| Self::get_cell(value, Self::AREA_CELL)),
      ))
      .or_default()
      .push(id);
    self.triangles.push(triangle);
  }

  /// How far apart, in texture repeats, two texture coordinates may be and be the same: the level stores them in 16
  /// bits and a byte.
  pub(crate) const UV_TOLERANCE: f64 = 0.004;

  /// Whether `level`'s corner `order[i]` stands within the tolerance of `placed`'s corner `i`, for each corner.
  fn is_same_order(level: &[DVec3; 3], placed: &[DVec3; 3], order: &[usize; 3]) -> bool {
    (0..3).all(|corner| level[order[corner]].distance_squared(placed[corner]) <= Self::TOLERANCE * Self::TOLERANCE)
  }

  /// Whether `level`'s texture coordinates, corner `order[i]` for the library's corner `i`, are the library's moved by
  /// one shift of whole repeats: xrLC moves a face's coordinates near the origin where its compiler shader optimises
  /// them.
  fn is_same_mapping(level: &[DVec2; 3], library: &[DVec2; 3], order: &[usize; 3]) -> bool {
    let shift: DVec2 = level[order[0]] - library[0];
    let whole: DVec2 = shift.round();

    (shift - whole).abs().max_element() <= Self::UV_TOLERANCE
      && (1..3).all(|corner| (level[order[corner]] - library[corner] - shift).abs().max_element() <= Self::UV_TOLERANCE)
  }

  /// Every order of a triangle's three corners.
  pub(crate) const PERMUTATIONS: [[usize; 3]; 6] = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];

  /// The cell of a triangle's two shorter edges over its longest, `edges` shortest first.
  fn get_ratios(edges: &[f64; 3]) -> [i64; 2] {
    [
      Self::get_cell(edges[0] / edges[2], Self::RATIO_STEP),
      Self::get_cell(edges[1] / edges[2], Self::RATIO_STEP),
    ]
  }

  fn get_cell(value: f64, size: f64) -> i64 {
    (value / size).floor() as i64
  }
}
