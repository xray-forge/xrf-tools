use glam::DVec2;

/// The convex hull of points in a plane by Andrew's monotone chain, as the SDK's fits outline a projection
/// (`ConvexHull2D::ByIncremental` with `RemoveCollinear`).
pub struct ConvexOutline;

impl ConvexOutline {
  /// The hull's corners counter-clockwise from the least by x then y, no point twice and none collinear with its
  /// neighbours. Fewer than three distinct points come back sorted, as they are.
  #[must_use]
  pub fn of(points: &[DVec2]) -> Vec<DVec2> {
    let mut sorted: Vec<DVec2> = points.to_vec();

    sorted.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    sorted.dedup();

    if sorted.len() < 3 {
      return sorted;
    }

    let turn = |o: DVec2, a: DVec2, b: DVec2| (a - o).perp_dot(b - o);
    let mut outline: Vec<DVec2> = Vec::with_capacity(sorted.len() + 1);

    // The lower chain left to right, then the upper chain back, never popping into the lower one.
    for pass in [sorted.clone(), sorted.into_iter().rev().collect()] {
      let start: usize = outline.len();

      for point in pass {
        while outline.len() >= start + 2 && turn(outline[outline.len() - 2], outline[outline.len() - 1], point) <= 0.0 {
          outline.pop();
        }

        outline.push(point);
      }

      outline.pop();
    }

    outline
  }
}
