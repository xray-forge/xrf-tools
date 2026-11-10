use glam::Vec4;

use crate::feature::scene::wind::wind_feature::WindFeature;
use crate::scene::level::level_time::LevelTime;
use crate::scene::static_scene::static_scene::StaticScene;

/// What of the interval a frame may come early by and still draw for the sway, so a 60 Hz display a hair short of an
/// interval does not halve the sway to every other frame.
const SWAY_SLACK: f32 = 0.1;

/// Texels a lean must cross before a map shows it.
const SWAY_TEXELS: f32 = 0.5;

/// How the trees sway this frame, as the shadows see it: how far they lean, and where they stand.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ShadowSway<'a> {
  /// The wind's amplitude, which a tree's reach is multiplied by to give how far it leans; zero while trees stand
  /// still.
  pub amplitude: f32,
  /// The farthest any tree reaches, in metres, weighted by its rigidity.
  pub reach: f32,
  /// The level's clock, which a map's last draw is timed by.
  pub time: LevelTime,
  /// The bounding sphere of every place that sways, and its own reach.
  pub places: &'a [(Vec4, f32)],
}

impl<'a> ShadowSway<'a> {
  /// Seconds between a shadow's draws for what sways alone, at most sixty a second: the sway is slow, and each draw
  /// over the trees costs a light face's or a cascade's cull and draw.
  pub(crate) const INTERVAL: f32 = 1.0 / 60.0;

  /// The sway the shadows see this frame at the level's clock `time`: the `wind`, the trees' reach in `scene`, and
  /// where they stand.
  pub(crate) fn of(scene: &'a StaticScene, wind: &WindFeature, time: LevelTime) -> Self {
    Self {
      amplitude: wind.get_sway_amplitude(),
      reach: scene.get_sway_reach(),
      time,
      places: scene.list_swaying(),
    }
  }

  /// Whether a map drawn at `drawn_at` draws again for the sway alone: once its interval passed, and only where a tree
  /// reaching `reach` leans across enough of a texel `texel` wide to show. Both are metres, or both are per metre of
  /// distance from a light.
  pub(crate) fn is_redrawn(&self, reach: f32, texel: f32, drawn_at: LevelTime) -> bool {
    self.is_redrawn_every(reach, texel, drawn_at, Self::INTERVAL)
  }

  /// `is_redrawn` with an interval of its own, `interval` seconds.
  pub(crate) fn is_redrawn_every(&self, reach: f32, texel: f32, drawn_at: LevelTime, interval: f32) -> bool {
    self.amplitude * reach > texel * SWAY_TEXELS
      && self
        .get_waited(drawn_at)
        .is_none_or(|waited| waited >= interval * (1.0 - SWAY_SLACK))
  }

  /// Whether a map drawn at `drawn_at` may wait yet for its next draw, for less than `most` seconds.
  pub(crate) fn can_wait(&self, drawn_at: LevelTime, most: f32) -> bool {
    self.get_waited(drawn_at).is_some_and(|waited| waited < most)
  }

  /// Seconds a map drawn at `drawn_at` has waited by the level's clock; none where it was drawn after the clock now
  /// stands, as a burst from an earlier time leaves every map: such a map is due, however near the clock reads.
  fn get_waited(&self, drawn_at: LevelTime) -> Option<f32> {
    (drawn_at <= self.time).then(|| self.time.get_since(drawn_at))
  }
}
