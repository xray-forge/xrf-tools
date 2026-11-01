use crate::feature::scene::grass::trace::traced_grass::TracedGrass;

/// A drawable kind's part of a view's scene trace as the frame recorded it, a variant a kind: what the trace's shaders
/// bind of it.
#[derive(Clone, Copy)]
pub enum DrawableKindTraced {
  /// The grass's planting, which covers a terrain hit, and the cards the view stands near its eye.
  Grass(TracedGrass),
}

impl DrawableKindTraced {
  /// The grass's part, which the trace's shaders bind by name; none for another kind's.
  pub fn get_grass(&self) -> Option<TracedGrass> {
    match *self {
      Self::Grass(grass) => Some(grass),
    }
  }
}
