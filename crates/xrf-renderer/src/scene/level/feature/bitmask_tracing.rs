use crate::feature::tracing::view_trace::traced_scene::TracedScene;

/// How the bitmask search meets the frame's tracing: the scene's trace, where it is built, and whether the indirect
/// light is traced in the search's place.
#[derive(Clone, Copy)]
pub struct BitmaskTracing<'a> {
  pub traced: Option<TracedScene<'a>>,
  pub is_light_traced: bool,
}
