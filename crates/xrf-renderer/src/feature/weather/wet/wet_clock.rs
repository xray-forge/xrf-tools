use crate::scene::level::view::view_time::ViewTime;

/// The clocks the wet surfaces run on this frame: the seconds their splashes, flow and ripples repeat by (the falling
/// rain's clock while it falls, the level's otherwise), and the view's time, whose steps their drift sums.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WetClock {
  pub splashes: f64,
  pub view: ViewTime,
}
