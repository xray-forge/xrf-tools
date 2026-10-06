use glam::Vec4;

/// What the enhanced foliage motion reads this frame, copied into the trees' `Wind` and the grass's `GrassWind`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FoliageWindValues {
  /// The way the wind blows across the engine's ground in `xy`, its speed in `z` and that speed's root in `w`.
  pub wind: Vec4,
  /// The grass's drift speed, turbulence, push and wave.
  pub grass: Vec4,
  /// The branches' drift speed, the trunks' swing speed and bend, and one while the enhanced motion draws.
  pub trees: Vec4,
  /// How far the flow fields have drifted, this frame and the last.
  pub anim: Vec4,
  pub previous_anim: Vec4,
  /// The wetness in `z`, and one in `w` while the flora is lit as foliage.
  pub flora: Vec4,
}
