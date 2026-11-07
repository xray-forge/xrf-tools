use crate::object::compile::skeleton::skeleton_vertex::SkeletonVertex;

/// Orders a child's triangles for the GPU's post-transform vertex cache, and its vertices by first use, where the
/// Actor Editor's export took D3DX's closed `D3DXOptimizeFaces` and `D3DXOptimizeVertices` (`MakeStripify`).
///
/// The triangles follow Tom Forsyth's "Linear-Speed Vertex Cache Optimisation" (2006): each vertex scores by where it
/// stands in a modelled LRU cache and by how few triangles still need it, each triangle by the sum of its corners, and
/// the best triangle among those touching the cache goes next, in time linear in the triangles. It reaches the cache
/// efficiency D3DX's order has on vanilla models without depending on it, and gives one order for one input.
pub(crate) struct VertexCacheOrder;

impl VertexCacheOrder {
  /// The modelled LRU cache's size, Forsyth's general choice, larger than the hardware FIFOs the order is measured on.
  const CACHE_SIZE: usize = 32;

  /// How fast a vertex's score falls with its age in the cache.
  const CACHE_DECAY_POWER: f32 = 1.5;

  /// The score of the last triangle's vertices, kept below that of the next ones so the order does not double back.
  const LAST_TRIANGLE_SCORE: f32 = 0.75;

  /// The boost for a vertex few triangles still need, so the order finishes off a region before leaving it.
  const VALENCE_BOOST_SCALE: f32 = 2.0;
  const VALENCE_BOOST_POWER: f32 = 0.5;

  /// The vertices and faces of one child, the faces reordered for the cache and the vertices renumbered in the order
  /// the faces first use them, so the GPU fetches them in sequence.
  pub(crate) fn apply(vertices: Vec<SkeletonVertex>, faces: &[[u16; 3]]) -> (Vec<SkeletonVertex>, Vec<[u16; 3]>) {
    let ordered: Vec<[u16; 3]> = Self::order_faces(faces, vertices.len());
    let mut new_index: Vec<Option<u16>> = vec![None; vertices.len()];
    let mut order: Vec<usize> = Vec::with_capacity(vertices.len());

    for vertex in ordered.iter().flatten() {
      let slot: &mut Option<u16> = &mut new_index[usize::from(*vertex)];

      if slot.is_none() {
        *slot = Some(order.len() as u16);
        order.push(usize::from(*vertex));
      }
    }

    // A vertex no face uses keeps a place after the used ones.
    for (vertex, slot) in new_index.iter_mut().enumerate() {
      if slot.is_none() {
        *slot = Some(order.len() as u16);
        order.push(vertex);
      }
    }

    let mut taken: Vec<Option<SkeletonVertex>> = vertices.into_iter().map(Some).collect();
    let reordered: Vec<SkeletonVertex> = order
      .iter()
      .map(|vertex| taken[*vertex].take().expect("each vertex is placed once"))
      .collect();
    let faces: Vec<[u16; 3]> = ordered
      .iter()
      .map(|face| face.map(|vertex| new_index[usize::from(vertex)].expect("every vertex is renumbered")))
      .collect();

    (reordered, faces)
  }

  /// The faces in cache order.
  pub(crate) fn order_faces(faces: &[[u16; 3]], vertex_count: usize) -> Vec<[u16; 3]> {
    if faces.len() <= 1 {
      return faces.to_vec();
    }

    // Each vertex's triangles, the ones not yet emitted kept at the front of its run.
    let mut remaining: Vec<usize> = vec![0; vertex_count];

    for vertex in faces.iter().flatten() {
      remaining[usize::from(*vertex)] += 1;
    }

    let mut offsets: Vec<usize> = Vec::with_capacity(vertex_count + 1);

    offsets.push(0);

    for count in &remaining {
      offsets.push(offsets.last().copied().unwrap_or_default() + count);
    }

    let mut adjacency: Vec<usize> = vec![0; offsets[vertex_count]];
    let mut filled: Vec<usize> = offsets[..vertex_count].to_vec();

    for (triangle, face) in faces.iter().enumerate() {
      for vertex in face {
        let vertex: usize = usize::from(*vertex);

        adjacency[filled[vertex]] = triangle;
        filled[vertex] += 1;
      }
    }

    let mut cache_position: Vec<Option<usize>> = vec![None; vertex_count];
    let mut vertex_score: Vec<f32> = (0..vertex_count)
      .map(|vertex| Self::vertex_score(None, remaining[vertex]))
      .collect();
    let mut triangle_score: Vec<f32> = faces
      .iter()
      .map(|face| face.iter().map(|vertex| vertex_score[usize::from(*vertex)]).sum())
      .collect();
    let mut is_emitted: Vec<bool> = vec![false; faces.len()];
    let mut cache: Vec<usize> = Vec::with_capacity(Self::CACHE_SIZE + 3);
    let mut ordered: Vec<[u16; 3]> = Vec::with_capacity(faces.len());
    let mut best: Option<usize> = Self::best_unemitted(&triangle_score, &is_emitted);

    while let Some(triangle) = best {
      let face: [u16; 3] = faces[triangle];

      ordered.push(face);
      is_emitted[triangle] = true;

      // The triangle leaves each of its vertices' runs of triangles still to go.
      for vertex in face.map(usize::from) {
        let run: &mut [usize] = &mut adjacency[offsets[vertex]..offsets[vertex] + remaining[vertex]];

        if let Some(position) = run.iter().position(|it| *it == triangle) {
          let last: usize = run.len() - 1;

          run.swap(position, last);
          remaining[vertex] -= 1;
        }
      }

      // The triangle's vertices go to the front of the cache, the rest move back, and those past its end fall out.
      let mut next: Vec<usize> = Vec::with_capacity(cache.len() + 3);

      for vertex in face.map(usize::from) {
        if !next.contains(&vertex) {
          next.push(vertex);
        }
      }

      next.extend(cache.iter().copied().filter(|vertex| !face.contains(&(*vertex as u16))));

      for (position, vertex) in next.iter().enumerate() {
        cache_position[*vertex] = (position < Self::CACHE_SIZE).then_some(position);
        vertex_score[*vertex] = Self::vertex_score(cache_position[*vertex], remaining[*vertex]);
      }

      // The next triangle is the best one around the cache, those just pushed out of it rescored too.
      best = None;

      let mut best_score: f32 = f32::MIN;

      for vertex in &next {
        for other in &adjacency[offsets[*vertex]..offsets[*vertex] + remaining[*vertex]] {
          let score: f32 = faces[*other].iter().map(|it| vertex_score[usize::from(*it)]).sum();

          triangle_score[*other] = score;

          if score > best_score {
            best_score = score;
            best = Some(*other);
          }
        }
      }

      next.truncate(Self::CACHE_SIZE);
      cache = next;

      // A region finished, so the next starts at the best triangle anywhere.
      if best.is_none() {
        best = Self::best_unemitted(&triangle_score, &is_emitted);
      }
    }

    ordered
  }

  fn best_unemitted(triangle_score: &[f32], is_emitted: &[bool]) -> Option<usize> {
    (0..triangle_score.len())
      .filter(|triangle| !is_emitted[*triangle])
      .max_by(|first, second| {
        triangle_score[*first]
          .total_cmp(&triangle_score[*second])
          .then(second.cmp(first))
      })
  }

  /// A vertex's score: how recently the cache took it, and how few triangles still need it. One no triangle needs
  /// scores nothing more.
  fn vertex_score(cache_position: Option<usize>, remaining: usize) -> f32 {
    if remaining == 0 {
      return -1.0;
    }

    let cache_score: f32 = match cache_position {
      None => 0.0,
      Some(position) if position < 3 => Self::LAST_TRIANGLE_SCORE,
      Some(position) => {
        let scale: f32 = 1.0 / (Self::CACHE_SIZE - 3) as f32;

        (1.0 - (position - 3) as f32 * scale).powf(Self::CACHE_DECAY_POWER)
      }
    };

    cache_score + Self::VALENCE_BOOST_SCALE * (remaining as f32).powf(-Self::VALENCE_BOOST_POWER)
  }
}
