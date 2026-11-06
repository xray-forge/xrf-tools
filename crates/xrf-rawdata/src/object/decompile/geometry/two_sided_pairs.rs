use std::collections::HashMap;

use crate::object::decompile::geometry::welded_face::WeldedFace;

/// Folds the mirrored copies a two-sided surface's export wrote back into single faces.
///
/// `CExportObjectOGF::PrepareMESH` and its skeleton counterpart write each face of an `sf2Sided` surface twice, the
/// second with its winding reversed and its normals inverted. A face and its mirror in the same OGF surface are one
/// authored face, kept once and marked two-sided.
///
/// Which of the two is kept decides how the face meets its neighbours: smoothing joins faces that run a shared edge in
/// opposite directions. A progressive mesh reorders the triangles, so neither copy is reliably first; the kept member
/// of each pair is the one consistent with a single-sided neighbour where it has one, and with the pairs already
/// chosen around it otherwise.
pub(crate) struct TwoSidedPairs;

impl TwoSidedPairs {
  pub(crate) fn fold(faces: Vec<WeldedFace>) -> Vec<WeldedFace> {
    let pairs: Vec<[usize; 2]> = Self::find_pairs(&faces);
    let kept: Vec<usize> = Self::orient(&faces, &pairs);
    let mut is_dropped: Vec<bool> = vec![false; faces.len()];
    let mut is_two_sided: Vec<bool> = vec![false; faces.len()];

    for (members, member) in pairs.iter().zip(kept) {
      is_two_sided[members[member]] = true;
      is_dropped[members[1 - member]] = true;
    }

    faces
      .into_iter()
      .enumerate()
      .filter(|(index, _)| !is_dropped[*index])
      .map(|(index, face)| WeldedFace {
        is_two_sided: is_two_sided[index],
        ..face
      })
      .collect()
  }

  /// Each face and its mirror, the face written first leading.
  fn find_pairs(faces: &[WeldedFace]) -> Vec<[usize; 2]> {
    let mut faces_by_points: HashMap<[u32; 4], Vec<usize>> = HashMap::new();

    for (index, face) in faces.iter().enumerate() {
      faces_by_points.entry(Self::get_bucket(face)).or_default().push(index);
    }

    let mut is_paired: Vec<bool> = vec![false; faces.len()];
    let mut pairs: Vec<[usize; 2]> = Vec::new();

    for (index, face) in faces.iter().enumerate() {
      if is_paired[index] {
        continue;
      }

      let mirror: Option<usize> = faces_by_points[&Self::get_bucket(face)]
        .iter()
        .copied()
        .find(|other| *other > index && !is_paired[*other] && face.is_mirrored_by(&faces[*other]));

      if let Some(mirror) = mirror {
        is_paired[index] = true;
        is_paired[mirror] = true;
        pairs.push([index, mirror]);
      }
    }

    pairs
  }

  /// Which member of each pair to keep, 0 or 1, so the kept faces run their shared edges in opposite directions with
  /// their neighbours.
  fn orient(faces: &[WeldedFace], pairs: &[[usize; 2]]) -> Vec<usize> {
    let mut pair_of_face: HashMap<usize, usize> = HashMap::new();

    for (pair, members) in pairs.iter().enumerate() {
      pair_of_face.insert(members[0], pair);
      pair_of_face.insert(members[1], pair);
    }

    let mut faces_of_edge: HashMap<[u32; 2], Vec<usize>> = HashMap::new();

    for (index, face) in faces.iter().enumerate() {
      for edge in 0..3 {
        faces_of_edge
          .entry(Self::get_edge_key(face, edge))
          .or_default()
          .push(index);
      }
    }

    let mut kept: Vec<Option<usize>> = vec![None; pairs.len()];
    let mut pending: Vec<usize> = Vec::new();

    // Single-sided neighbours anchor the pairs touching them.
    for (pair, members) in pairs.iter().enumerate() {
      let anchor: Option<usize> = (0..3).find_map(|edge| {
        let (start, end) = Self::get_edge(&faces[members[0]], edge);

        faces_of_edge[&Self::get_edge_key(&faces[members[0]], edge)]
          .iter()
          .find(|neighbour| !pair_of_face.contains_key(*neighbour))
          .and_then(|neighbour| Self::find_member_running(faces, members, &faces[*neighbour], start, end))
      });

      if let Some(member) = anchor {
        kept[pair] = Some(member);
        pending.push(pair);
      }
    }

    for seed in 0..pairs.len() {
      if kept[seed].is_none() {
        kept[seed] = Some(0);
        pending.push(seed);
      }

      while let Some(pair) = pending.pop() {
        let face: &WeldedFace = &faces[pairs[pair][kept[pair].expect("a pending pair is oriented")]];

        for edge in 0..3 {
          let (start, end) = Self::get_edge(face, edge);

          for neighbour in &faces_of_edge[&Self::get_edge_key(face, edge)] {
            let Some(other) = pair_of_face.get(neighbour).copied() else {
              continue;
            };

            if kept[other].is_some() {
              continue;
            }

            // The neighbour runs this edge the other way round, from `end` to `start`.
            kept[other] = Some(
              (0..2)
                .find(|member| Self::runs(&faces[pairs[other][*member]], end, start))
                .unwrap_or(0),
            );
            pending.push(other);
          }
        }
      }
    }

    kept.into_iter().map(|member| member.unwrap_or(0)).collect()
  }

  /// The member of `members` that runs the edge from `start` to `end` opposite to `neighbour`, which shares it.
  fn find_member_running(
    faces: &[WeldedFace],
    members: &[usize; 2],
    neighbour: &WeldedFace,
    start: u32,
    end: u32,
  ) -> Option<usize> {
    let (from, to) = if Self::runs(neighbour, start, end) {
      (end, start)
    } else {
      (start, end)
    };

    (0..2).find(|member| Self::runs(&faces[members[*member]], from, to))
  }

  /// Whether `face` runs one of its edges from `start` to `end`.
  fn runs(face: &WeldedFace, start: u32, end: u32) -> bool {
    (0..3).any(|edge| Self::get_edge(face, edge) == (start, end))
  }

  /// Edge `edge` of `face`, from corner `edge` to corner `(edge + 1) % 3`.
  fn get_edge(face: &WeldedFace, edge: usize) -> (u32, u32) {
    (face.points[edge], face.points[(edge + 1) % 3])
  }

  /// The edge's points regardless of direction.
  fn get_edge_key(face: &WeldedFace, edge: usize) -> [u32; 2] {
    let (start, end) = Self::get_edge(face, edge);

    if start <= end { [start, end] } else { [end, start] }
  }

  /// The surface and the points regardless of order, which a face and its mirror share.
  fn get_bucket(face: &WeldedFace) -> [u32; 4] {
    let mut points: [u32; 3] = face.points;

    points.sort_unstable();

    let surface: u32 = u32::try_from(face.surface).unwrap_or(u32::MAX);

    [surface, points[0], points[1], points[2]]
  }
}
