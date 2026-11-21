use std::collections::HashMap;

use crate::level::build::level_build_part_face::LevelBuildPartFace;

/// The parts of a build's flagged faces, `CGeomPartExtractor`'s grouping, with the faces around each welded point they
/// were grouped by, kept so each part's neighbours are found without walking the whole build again.
pub(crate) struct LevelBuildPartGroups {
  /// Each part's faces, in the order `recurse_tri` reaches them.
  pub(crate) parts: Vec<Vec<usize>>,
  /// Each welded point's faces, in the order they were added.
  around: Vec<Vec<usize>>,
}

impl LevelBuildPartGroups {
  /// The parts of `faces`, whose points number `point_count`: each depth first from the first face no part holds yet,
  /// through the faces around each point in the order they were added.
  pub(crate) fn of(faces: &[LevelBuildPartFace], point_count: usize) -> Self {
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); point_count];

    for (index, face) in faces.iter().enumerate() {
      for point in face.points {
        around[point as usize].push(index);
      }
    }

    let mut is_taken: Vec<bool> = vec![false; faces.len()];
    let mut parts: Vec<Vec<usize>> = Vec::new();

    for start in 0..faces.len() {
      if is_taken[start] {
        continue;
      }

      let mut part: Vec<usize> = Vec::new();
      // Each frame a face whose corners are walked, and how far.
      let mut stack: Vec<(usize, usize, usize)> = vec![(start, 0, 0)];

      is_taken[start] = true;
      part.push(start);

      while let Some((face, corner, next)) = stack.pop() {
        if corner == 3 {
          continue;
        }

        let reachable: &[usize] = &around[faces[face].points[corner] as usize];

        if next == reachable.len() {
          stack.push((face, corner + 1, 0));
          continue;
        }

        stack.push((face, corner, next + 1));

        let reached: usize = reachable[next];

        if !is_taken[reached] {
          is_taken[reached] = true;
          part.push(reached);
          stack.push((reached, 0, 0));
        }
      }

      parts.push(part);
    }

    Self { parts, around }
  }

  /// Each face of `part`'s neighbours across an edge, as places in `part`, once each, in the order `SBPart::prepare`
  /// lists them. A face sharing an edge shares its points, so it is always of the same part.
  pub(crate) fn get_edge_neighbours(&self, faces: &[LevelBuildPartFace], part: &[usize]) -> Vec<Vec<usize>> {
    let places: HashMap<usize, usize> = part.iter().enumerate().map(|(place, face)| (*face, place)).collect();
    let mut neighbours: Vec<Vec<usize>> = vec![Vec::new(); part.len()];

    for (place, face) in part.iter().enumerate() {
      let points: [u32; 3] = faces[*face].points;

      for point in points {
        for other in &self.around[point as usize] {
          let shared: usize = faces[*other]
            .points
            .iter()
            .map(|it| points.iter().filter(|point| *point == it).count())
            .sum();

          if *other == *face || shared < 2 {
            continue;
          }

          let other_place: usize = places[other];

          for (from, to) in [(place, other_place), (other_place, place)] {
            if !neighbours[from].contains(&to) {
              neighbours[from].push(to);
            }
          }
        }
      }
    }

    neighbours
  }
}
