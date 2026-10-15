use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::Range;

use glam::{Vec3, Vec4};
use xrf_visual::{SectorGeometry, VisualClusters};

use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::static_geometry::StaticGeometry;

/// A geometry's clusters read once for every range cut from them: the packer's ranges and spheres, where each cluster
/// starts, and the indices and positions a range without clusters is cut from, read the first time one is.
pub struct GeometryClusters<'a> {
  geometry: &'a SectorGeometry,
  buffer: &'a [u8],
  ranges: Vec<[u32; 4]>,
  spheres: Vec<Vec4>,
  /// The clusters starting at each index, by their place in `ranges`, in order.
  starts: HashMap<u32, Vec<usize>>,
  vertices: OnceCell<(Vec<u32>, Vec<[f32; 3]>)>,
}

impl<'a> GeometryClusters<'a> {
  pub fn new(geometry: &'a SectorGeometry, buffer: &'a [u8]) -> Self {
    let ranges: Vec<[u32; 4]> = read_pods(buffer, &geometry.clusters.ranges);
    let mut starts: HashMap<u32, Vec<usize>> = HashMap::new();

    for (index, range) in ranges.iter().enumerate() {
      starts.entry(range[0]).or_default().push(index);
    }

    Self {
      geometry,
      buffer,
      spheres: read_pods(buffer, &geometry.clusters.spheres),
      ranges,
      starts,
      vertices: OnceCell::new(),
    }
  }

  /// The clusters covering an index range, as `(first index, triangles, sphere)`: the packer's own where they tile it
  /// exactly, else the range cut in runs of `CLUSTER_TRIANGLES` with spheres taken from their vertices.
  pub fn cut(&self, start: u32, count: u32) -> Vec<(u32, u32, Vec4)> {
    // A pack whose spheres are fewer than its ranges is cut anew, as one without clusters is.
    if let Some(run) = self.find_run(start, count)
      && let Some(run_spheres) = self.spheres.get(run.clone())
    {
      return self.ranges[run]
        .iter()
        .zip(run_spheres)
        .map(|(range, sphere)| (range[0], range[1], *sphere))
        .collect();
    }

    let (indices, positions) = self.vertices.get_or_init(|| {
      (
        read_pods(self.buffer, &self.geometry.indices),
        read_pods(self.buffer, &self.geometry.positions),
      )
    });
    let end: u32 = start.saturating_add(count);

    (start..end)
      .step_by(StaticGeometry::CLUSTER_VERTICES as usize)
      .map(|first| {
        let triangles: u32 = ((end - first) / 3).min(StaticGeometry::CLUSTER_TRIANGLES);
        let taken: &[u32] = indices
          .get(first as usize..)
          .map_or(&[], |rest| &rest[..rest.len().min((triangles * 3) as usize)]);
        let points: Vec<Vec3> = taken
          .iter()
          .filter_map(|index| positions.get(*index as usize).copied().map(Vec3::from_array))
          .collect();

        (first, triangles, Self::bound_points(&points))
      })
      .collect()
  }

  /// The clusters, by their place in `ranges`, of the run of `count` indices at `start`, among those starting there.
  fn find_run(&self, start: u32, count: u32) -> Option<Range<usize>> {
    VisualClusters::find_run_among(&self.ranges, self.starts.get(&start)?.iter().copied(), start, count)
  }

  /// A sphere holding every point: centred on their box, as far out as the farthest.
  fn bound_points(points: &[Vec3]) -> Vec4 {
    let (low, high): (Vec3, Vec3) = points.iter().fold((Vec3::MAX, Vec3::MIN), |(low, high), point| {
      (low.min(*point), high.max(*point))
    });
    let center: Vec3 = (low + high) * 0.5;
    let radius: f32 = points.iter().map(|point| point.distance(center)).fold(0.0, f32::max);

    if points.is_empty() {
      Vec4::new(0.0, 0.0, 0.0, -1.0)
    } else {
      center.extend(radius)
    }
  }
}
