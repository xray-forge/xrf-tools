use glam::Vec4;

/// Seconds between a shadow's draws for what sways alone, at most sixty a second: the sway is slow, and each draw over
/// the trees costs a light face's or a cascade's cull and draw.
pub const SHADOW_SWAY_INTERVAL: f32 = 1.0 / 60.0;

/// What of the interval a frame may come early by and still draw for the sway, so a 60 Hz display a hair short of an
/// interval does not halve the sway to every other frame.
const SWAY_SLACK: f32 = 0.1;

/// Texels a lean must cross before a map shows it.
const SWAY_TEXELS: f32 = 0.5;

/// How the trees sway this frame, as the shadows see it: how far they lean, and where they stand.
#[derive(Clone, Copy, Debug)]
pub struct ShadowSway<'a> {
  /// The wind's amplitude, which a tree's reach is multiplied by to give how far it leans; zero while trees stand still.
  pub amplitude: f32,
  /// The farthest any tree reaches, in metres, weighted by its rigidity.
  pub reach: f32,
  /// Seconds the sway runs by, which a map's last draw is timed in.
  pub time: f32,
  /// The bounding sphere of every place that sways, and its own reach.
  pub places: &'a [(Vec4, f32)],
}

impl ShadowSway<'_> {
  /// Whether a map drawn at `drawn_at` draws again for the sway alone: once its interval passed, and only where a tree
  /// reaching `reach` leans across enough of a texel `texel` wide to show. Both are metres, or both are per metre of
  /// distance from a light.
  pub fn is_redrawn(&self, reach: f32, texel: f32, drawn_at: f32) -> bool {
    self.amplitude * reach > texel * SWAY_TEXELS && self.time - drawn_at >= SHADOW_SWAY_INTERVAL * (1.0 - SWAY_SLACK)
  }
}
