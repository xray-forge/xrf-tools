/// One triangle of the shape being collapsed.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ProgressiveTriangle {
  pub(crate) corners: [u16; 3],
  /// The sliding window level it belongs to: a collapse's new triangles are on the level after its own.
  pub(crate) level: u32,
  pub(crate) is_alive: bool,
}

/// The mesh as collapses leave it: its triangles, the dead kept so an index names one triangle for good, and each
/// vertex's live triangles in the order they were made.
#[derive(Clone, Debug)]
pub(crate) struct ProgressiveShape {
  pub(crate) triangles: Vec<ProgressiveTriangle>,
  incident: Vec<Vec<u32>>,
}

impl ProgressiveShape {
  pub(crate) fn new(vertex_count: usize) -> Self {
    Self {
      triangles: Vec::new(),
      incident: vec![Vec::new(); vertex_count],
    }
  }

  /// Makes a triangle, live, on `level`.
  pub(crate) fn add(&mut self, corners: [u16; 3], level: u32) -> u32 {
    let index: u32 = u32::try_from(self.triangles.len()).expect("a shape's triangles fit 32-bit indices");

    self.triangles.push(ProgressiveTriangle {
      corners,
      level,
      is_alive: true,
    });

    for corner in corners {
      self.incident[usize::from(corner)].push(index);
    }

    index
  }

  pub(crate) fn remove(&mut self, triangle: u32) {
    let ProgressiveTriangle { corners, .. } = self.triangles[triangle as usize];

    self.triangles[triangle as usize].is_alive = false;

    for corner in corners {
      self.incident[usize::from(corner)].retain(|it| *it != triangle);
    }
  }

  /// The live triangles at `vertex`, in the order they were made.
  pub(crate) fn get_incident(&self, vertex: u16) -> &[u32] {
    &self.incident[usize::from(vertex)]
  }

  /// The vertices sharing an edge with `vertex`, in the order its triangles first name them.
  pub(crate) fn get_neighbours(&self, vertex: u16) -> Vec<u16> {
    let mut neighbours: Vec<u16> = Vec::new();

    for triangle in self.get_incident(vertex) {
      for corner in self.triangles[*triangle as usize].corners {
        if corner != vertex && !neighbours.contains(&corner) {
          neighbours.push(corner);
        }
      }
    }

    neighbours
  }

  /// Whether the edge from `first` to `second` has exactly a triangle on each side: one naming them in that order,
  /// one in the other.
  pub(crate) fn is_interior(&self, first: u16, second: u16) -> bool {
    let (mut forward, mut backward): (u32, u32) = (0, 0);

    for triangle in self.get_incident(first) {
      let corners: [u16; 3] = self.triangles[*triangle as usize].corners;

      for corner in 0..3 {
        let (from, to) = (corners[corner], corners[(corner + 1) % 3]);

        forward += u32::from(from == first && to == second);
        backward += u32::from(from == second && to == first);
      }
    }

    forward == 1 && backward == 1
  }

  /// Whether a live triangle other than those at `excluded` has the corners of `corners`, in any order.
  pub(crate) fn has_triangle_of(&self, corners: [u16; 3], excluded: u16) -> bool {
    let mut sorted: [u16; 3] = corners;

    sorted.sort_unstable();

    self.get_incident(corners[0]).iter().any(|triangle| {
      let other: [u16; 3] = self.triangles[*triangle as usize].corners;
      let mut other_sorted: [u16; 3] = other;

      other_sorted.sort_unstable();

      !other.contains(&excluded) && other_sorted == sorted
    })
  }

  /// The live triangles.
  pub(crate) fn get_alive(&self) -> impl Iterator<Item = u32> + '_ {
    (0..self.triangles.len() as u32).filter(|triangle| self.triangles[*triangle as usize].is_alive)
  }
}
