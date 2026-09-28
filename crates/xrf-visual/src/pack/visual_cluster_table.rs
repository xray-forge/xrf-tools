use crate::data::visual::geometry::visual_clusters::VisualClusters;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;

/// The clusters of one geometry under construction, cut from each run of its indices that is drawn: every packer's
/// one way of cutting them, so every static draw culls the same.
#[derive(Debug, Default)]
pub struct VisualClusterTable {
  ranges: Vec<u32>,
  spheres: Vec<f32>,
  /// The positions one cluster reaches, kept between clusters so cutting one allocates nothing.
  points: Vec<[f32; 3]>,
}

impl VisualClusterTable {
  /// Cuts a run of indices into clusters of consecutive triangles.
  ///
  /// # Arguments
  ///
  /// * `indices` - Every index of the geometry, which the run's start counts in.
  /// * `positions` - Every vertex's position, which the indices name.
  /// * `start` - The run's first index, on a triangle.
  /// * `count` - Indices in it, whole triangles.
  /// * `drawable` - The drawable it is of, or [`VisualClusters::NO_DRAWABLE`].
  ///
  /// A consumer finds a run's clusters by its start: the cluster starting there, and those after it that each start
  /// where the one before ends.
  pub fn push_run(&mut self, indices: &[u32], positions: &[[f32; 3]], start: u32, count: u32, drawable: u32) {
    let per_cluster: u32 = VisualClusters::MAX_TRIANGLES * 3;
    let mut at: u32 = start;

    while at < start + count {
      let length: u32 = per_cluster.min(start + count - at);
      let run: &[u32] = &indices[at as usize..(at + length) as usize];

      self.ranges.extend([at, length / 3, drawable, 0]);
      self.points.clear();
      self
        .points
        .extend(run.iter().filter_map(|index| positions.get(*index as usize)));
      self.spheres.extend(Self::get_sphere(&self.points));
      at += length;
    }
  }

  /// Clusters cut so far.
  pub fn get_count(&self) -> u32 {
    (self.ranges.len() / VisualClusters::RANGE_WORDS) as u32
  }

  /// Writes the table into the buffer and says where it landed.
  pub fn write_into(&self, builder: &mut VisualBufferBuilder) -> VisualClusters {
    VisualClusters {
      ranges: builder.push_u32_section(&self.ranges),
      spheres: builder.push_f32_section(&self.spheres),
    }
  }

  /// The smaller of two spheres holding a cluster's points: about the centre of the box they span, and Ritter's, grown
  /// from the two farthest apart; nought for a cluster reaching none.
  fn get_sphere(points: &[[f32; 3]]) -> [f32; 4] {
    let Some(first) = points.first() else {
      return [0.0; 4];
    };

    let mut min: [f32; 3] = *first;
    let mut max: [f32; 3] = *first;

    for point in points {
      for axis in 0..3 {
        min[axis] = min[axis].min(point[axis]);
        max[axis] = max[axis].max(point[axis]);
      }
    }

    let boxed: [f32; 3] = [0, 1, 2].map(|axis| (min[axis] + max[axis]) / 2.0);
    let boxed_radius: f32 = Self::get_reach(points, boxed);
    let (ritter, ritter_radius): ([f32; 3], f32) = Self::get_ritter(points);

    if ritter_radius < boxed_radius {
      [ritter[0], ritter[1], ritter[2], ritter_radius]
    } else {
      [boxed[0], boxed[1], boxed[2], boxed_radius]
    }
  }

  /// Ritter's sphere: from the point farthest from the first to the point farthest from that, grown over every point
  /// left outside it.
  fn get_ritter(points: &[[f32; 3]]) -> ([f32; 3], f32) {
    let far: [f32; 3] = Self::get_farthest(points, points[0]);
    let other: [f32; 3] = Self::get_farthest(points, far);
    let mut centre: [f32; 3] = [0, 1, 2].map(|axis| (far[axis] + other[axis]) / 2.0);
    let mut radius: f32 = Self::get_distance(far, other) / 2.0;

    for point in points {
      let distance: f32 = Self::get_distance(*point, centre);

      if distance > radius {
        let grown: f32 = (radius + distance) / 2.0;
        let shift: f32 = (grown - radius) / distance;

        centre = [0, 1, 2].map(|axis| centre[axis] + (point[axis] - centre[axis]) * shift);
        radius = grown;
      }
    }

    // Grown in float steps, it may fall short of a point by a rounding: it is made to reach all of them.
    (centre, Self::get_reach(points, centre).max(radius))
  }

  fn get_farthest(points: &[[f32; 3]], from: [f32; 3]) -> [f32; 3] {
    *points
      .iter()
      .max_by(|left, right| Self::get_distance(**left, from).total_cmp(&Self::get_distance(**right, from)))
      .unwrap_or(&from)
  }

  /// How far the farthest point stands from a centre.
  fn get_reach(points: &[[f32; 3]], centre: [f32; 3]) -> f32 {
    points
      .iter()
      .map(|point| Self::get_distance(*point, centre))
      .fold(0.0, f32::max)
  }

  fn get_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2) + (left[2] - right[2]).powi(2)).sqrt()
  }
}
