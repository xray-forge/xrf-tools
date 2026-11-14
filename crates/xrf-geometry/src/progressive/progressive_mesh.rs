use std::collections::HashSet;

use crate::mesh::vertex_cache_order::VertexCacheOrder;
use crate::progressive::progressive_collapse::ProgressiveCollapse;
use crate::progressive::progressive_point::ProgressivePoint;
use crate::progressive::progressive_quadric::ProgressiveQuadric;
use crate::progressive::progressive_shape::ProgressiveShape;
use crate::progressive::progressive_window::ProgressiveWindow;

/// A mesh made progressive by sliding windows, as xrLC's tools make one (`VIPM_Convert`, `ETools/PropSlimTools.cpp`,
/// after Tom Forsyth's "Comparison of VIPM Methods", Game Programming Gems 2, 2001).
///
/// Edges are collapsed one at a time, each time the one whose removed vertex, moved onto its neighbour, lies least far
/// off the planes of the triangles around it, measured by a quadric over position and texture coordinate
/// ([`ProgressiveQuadric`]). A vertex on an open edge never moves. The collapses fall into levels: the triangles a
/// collapse makes belong to the next level, and a collapse touching one must start that level, which it does only
/// where its error beats the best on the current level by more than the mean error ([`Self::LEVEL_TOLERANCE`]). Each
/// level's triangles are laid out once in the index buffer so every state of it is one contiguous run: those its
/// collapses will make first, then those it leaves alone, then those its collapses take away. The vertices are
/// renumbered so each state's are a prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressiveMesh {
  /// Where each vertex goes: its new index, by its old one.
  pub permutation: Vec<u16>,
  /// The index buffer, every level's run, in new vertex indices.
  pub faces: Vec<[u16; 3]>,
  /// One window a state, the whole mesh first and then one more for each collapse.
  pub windows: Vec<ProgressiveWindow>,
}

/// The collapse a pass found best: its error, the vertex removed and the one kept.
type Candidate = (f32, u16, u16);

impl ProgressiveMesh {
  /// The levels a mesh may reach, xrLC's `VIPM_Convert(25, ...)`.
  pub const MAXIMUM_LEVEL: u32 = 25;

  /// How much worse, in mean errors, the best collapse on the current level may be before one starting a new level
  /// is taken instead.
  pub const LEVEL_TOLERANCE: f32 = 1.0;

  /// No collapse found yet, `1.0e10f` as the source starts its search.
  const NO_ERROR: f32 = 1.0e10;

  /// The progressive form of `faces` over `points`, or none where it cannot be built: a mesh too large for 16-bit
  /// indices, two triangles on the same three corners, or windows the layout gets wrong. A mesh with no edge to
  /// collapse has the one window of the whole mesh.
  pub fn build(points: &[ProgressivePoint], faces: &[[u16; 3]]) -> Option<Self> {
    if points.len() >= usize::from(u16::MAX) || faces.len() >= usize::from(u16::MAX) || Self::has_repeat(faces) {
      return None;
    }

    let mut shape: ProgressiveShape = ProgressiveShape::new(points.len());

    for face in faces {
      shape.add(*face, 0);
    }

    let collapses: Vec<ProgressiveCollapse> = Self::collapse(&mut shape, points);

    Self::lay_out(&shape, &collapses, points.len())
  }

  /// How many collapses the windows step through.
  pub fn get_collapse_count(&self) -> usize {
    self.windows.len() - 1
  }

  /// Whether two faces stand on the same three corners in any order (`Object::Valid`).
  fn has_repeat(faces: &[[u16; 3]]) -> bool {
    let mut seen: HashSet<[u16; 3]> = HashSet::with_capacity(faces.len());

    faces.iter().any(|face| {
      let mut sorted: [u16; 3] = *face;

      sorted.sort_unstable();

      !seen.insert(sorted)
    })
  }

  /// Collapses edges until none is allowed or the levels run out (`CalculateAllCollapses`), each vertex's candidate
  /// errors kept until a collapse changes the triangles around it.
  fn collapse(shape: &mut ProgressiveShape, points: &[ProgressivePoint]) -> Vec<ProgressiveCollapse> {
    let positions: Vec<[f64; ProgressiveQuadric::DIMENSIONS]> =
      points.iter().map(|point| point.to_quadric_point()).collect();
    let mut errors: Vec<Option<Vec<(u16, f32)>>> = vec![None; points.len()];
    let mut collapses: Vec<ProgressiveCollapse> = Vec::new();
    let mut level: u32 = 0;

    loop {
      let mut best: Option<Candidate> = None;
      let mut best_new_level: Option<Candidate> = None;
      let (mut sum, mut count): (f32, u32) = (0.0, 0);

      for vertex in 0..points.len() as u16 {
        let candidates: &Vec<(u16, f32)> =
          errors[usize::from(vertex)].get_or_insert_with(|| Self::get_errors(shape, &positions, vertex));

        if candidates.is_empty() {
          continue;
        }

        let needs_new_level: bool = shape
          .get_incident(vertex)
          .iter()
          .any(|triangle| shape.triangles[*triangle as usize].level != level);

        for (neighbour, error) in candidates {
          sum += error;
          count += 1;

          let slot: &mut Option<Candidate> = if needs_new_level {
            &mut best_new_level
          } else {
            &mut best
          };

          if slot.is_none_or(|(held, ..)| held > *error) && Self::is_allowed(shape, vertex, *neighbour) {
            *slot = Some((*error, vertex, *neighbour));
          }
        }
      }

      let average: f32 = sum / count.max(1) as f32;
      let best_error: f32 = best.map_or(Self::NO_ERROR, |(error, ..)| error);
      let new_level_error: f32 = best_new_level.map_or(Self::NO_ERROR, |(error, ..)| error);

      if best_error > new_level_error + average * Self::LEVEL_TOLERANCE {
        best = best_new_level;
      }

      let Some((_, binned, kept)) = best else {
        break;
      };
      let collapse: ProgressiveCollapse = Self::collapse_edge(shape, binned, kept, &mut level);

      for triangle in collapse.removed.iter().chain(&collapse.added) {
        for corner in shape.triangles[*triangle as usize].corners {
          errors[usize::from(corner)] = None;
        }
      }

      collapses.push(collapse);

      if level > Self::MAXIMUM_LEVEL {
        break;
      }
    }

    collapses
  }

  /// The error of moving `vertex` onto each neighbour, none where it stands on an open edge or on no edge at all
  /// (`FindCollapseError`).
  fn get_errors(
    shape: &ProgressiveShape,
    positions: &[[f64; ProgressiveQuadric::DIMENSIONS]],
    vertex: u16,
  ) -> Vec<(u16, f32)> {
    let neighbours: Vec<u16> = shape.get_neighbours(vertex);

    if neighbours
      .iter()
      .any(|neighbour| !shape.is_interior(vertex, *neighbour))
    {
      return Vec::new();
    }

    let mut quadric: ProgressiveQuadric = ProgressiveQuadric::ZERO;

    for triangle in shape.get_incident(vertex) {
      let [first, second, third] = shape.triangles[*triangle as usize].corners;

      quadric.add(&ProgressiveQuadric::of_triangle(
        &positions[usize::from(first)],
        &positions[usize::from(second)],
        &positions[usize::from(third)],
      ));
    }

    neighbours
      .into_iter()
      .map(|neighbour| (neighbour, quadric.evaluate(&positions[usize::from(neighbour)]) as f32))
      .collect()
  }

  /// Whether moving `binned` onto `kept` leaves no two triangles on the same three corners, which would fold the
  /// surface onto itself.
  fn is_allowed(shape: &ProgressiveShape, binned: u16, kept: u16) -> bool {
    let mut made: HashSet<[u16; 3]> = HashSet::new();

    shape.get_incident(binned).iter().all(|triangle| {
      let corners: [u16; 3] = shape.triangles[*triangle as usize].corners;

      if corners.contains(&kept) {
        return true;
      }

      let moved: [u16; 3] = corners.map(|corner| if corner == binned { kept } else { corner });
      let mut sorted: [u16; 3] = moved;

      sorted.sort_unstable();

      made.insert(sorted) && !shape.has_triangle_of(moved, binned)
    })
  }

  /// Moves `binned` onto `kept` (`Object::CreateEdgeCollapse`): first starting a new level where a triangle at the
  /// vertex is on the next one, then taking away every triangle at it and making again, on the next level, those
  /// that did not hold the collapsed edge.
  fn collapse_edge(shape: &mut ProgressiveShape, binned: u16, kept: u16, level: &mut u32) -> ProgressiveCollapse {
    let incident: Vec<u32> = shape.get_incident(binned).to_vec();

    if incident
      .iter()
      .any(|triangle| shape.triangles[*triangle as usize].level != *level)
    {
      *level += 1;

      for triangle in shape.get_alive().collect::<Vec<u32>>() {
        shape.triangles[triangle as usize].level = *level;
      }
    }

    let (closed, remapped): (Vec<u32>, Vec<u32>) = incident
      .into_iter()
      .partition(|triangle| shape.triangles[*triangle as usize].corners.contains(&kept));

    for triangle in remapped.iter().chain(&closed) {
      shape.remove(*triangle);
    }

    let added: Vec<u32> = remapped
      .iter()
      .map(|triangle| {
        let corners: [u16; 3] = shape.triangles[*triangle as usize]
          .corners
          .map(|corner| if corner == binned { kept } else { corner });

        shape.add(corners, *level + 1)
      })
      .collect();

    ProgressiveCollapse {
      binned,
      level: *level,
      removed: remapped.into_iter().chain(closed).collect(),
      added,
    }
  }

  /// The index buffer and windows of the collapses (`CalculateSW`, `ETools/object_sliding.cpp`): the vertices
  /// renumbered so the last collapse's removed vertex is last, then each level from the coarsest laid out as a run
  /// every state of it is a window of, each group of triangles in it in vertex cache order.
  fn lay_out(shape: &ProgressiveShape, collapses: &[ProgressiveCollapse], vertex_count: usize) -> Option<Self> {
    let count: usize = collapses.len();
    let mut permutation: Vec<Option<u16>> = vec![None; vertex_count];

    for (index, collapse) in collapses.iter().enumerate() {
      permutation[usize::from(collapse.binned)] = Some((vertex_count - 1 - index) as u16);
    }

    let mut next: u16 = 0;

    for slot in &mut permutation {
      if slot.is_none() {
        *slot = Some(next);
        next += 1;
      }
    }

    let permutation: Vec<u16> = permutation.into_iter().flatten().collect();
    let renumbered = |triangle: u32| {
      shape.triangles[triangle as usize]
        .corners
        .map(|corner| permutation[usize::from(corner)])
    };
    let mut alive: Vec<bool> = shape.triangles.iter().map(|triangle| triangle.is_alive).collect();
    let mut alive_count: usize = alive.iter().filter(|it| **it).count();
    let mut faces: Vec<[u16; 3]> = Vec::new();
    let mut windows: Vec<Option<ProgressiveWindow>> = vec![None; count + 1];
    let window = |offset: usize, triangles: usize, collapsed: usize| ProgressiveWindow {
      offset: (offset * 3) as u32,
      triangles: triangles as u16,
      vertices: (vertex_count - collapsed) as u16,
    };

    windows[count] = Some(window(0, alive_count, count));

    let push = |faces: &mut Vec<[u16; 3]>, group: &[u32]| {
      let renumbered: Vec<[u16; 3]> = group.iter().map(|triangle| renumbered(*triangle)).collect();

      faces.extend(VertexCacheOrder::order_faces(&renumbered, vertex_count));
    };
    let top: u32 = collapses.last().map_or(0, |collapse| collapse.level);
    let mut last: usize = count;

    for level in (0..=top).rev() {
      let first: usize = collapses[..last]
        .iter()
        .rposition(|collapse| collapse.level != level)
        .map_or(0, |it| it + 1);
      let members: &[ProgressiveCollapse] = &collapses[first..last];
      let made: HashSet<u32> = members
        .iter()
        .flat_map(|collapse| collapse.added.iter().copied())
        .collect();

      // The triangles its collapses make, the last collapse's first; a lower level's last collapse leaves its own
      // out, the state before it being the window the level above already holds.
      let mut offsets: Vec<usize> = vec![0; members.len()];
      let mut offset: usize = faces.len();

      for (position, collapse) in members.iter().enumerate().rev() {
        if position + 1 < members.len() || level == top {
          push(&mut faces, &collapse.added);
          offset += collapse.added.len();
        }

        offsets[position] = offset;
      }

      // Those its collapses leave alone, then those they take away.
      let untouched: Vec<u32> = (0..alive.len() as u32)
        .filter(|triangle| alive[*triangle as usize] && !made.contains(triangle))
        .collect();

      push(&mut faces, &untouched);

      for collapse in members.iter().rev() {
        push(&mut faces, &collapse.removed);
      }

      // Each state of the level, undoing its collapses from the last.
      for (position, collapse) in members.iter().enumerate().rev() {
        for triangle in &collapse.added {
          alive[*triangle as usize] = false;
        }

        for triangle in &collapse.removed {
          alive[*triangle as usize] = true;
        }

        alive_count = alive_count + collapse.removed.len() - collapse.added.len();
        windows[first + position] = Some(window(offsets[position], alive_count, first + position));
      }

      last = first;
    }

    let windows: Vec<ProgressiveWindow> = windows.into_iter().collect::<Option<_>>()?;
    let reaches = windows.iter().all(|window| {
      let start: usize = window.offset as usize / 3;

      faces
        .get(start..start + usize::from(window.triangles))
        .is_some_and(|run| run.iter().flatten().all(|vertex| *vertex < window.vertices))
    });

    reaches.then_some(Self {
      permutation,
      faces,
      windows,
    })
  }
}
