use glam::Vec4;

use crate::feature::weather::water::enhanced::water_flow::WaterFlow;
use crate::scene::level::level_time::LevelTime;

/// Where each of the enhanced water's map reads stands this frame, as `shaders/static/water_enhanced.wgsl` adds it to
/// its unscrolled coordinates: the flow's sums multiplied out in `f64` and taken within the map's repeat of one, so
/// each read moves as finely after a day as in a level's first minute, where the sums themselves as `f32` would step
/// and stall. The waves' scroll moves every read off the waves' coordinates by its own share, turned and tiled as that
/// read is, so it is worked out here for each read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct EnhancedWaterScroll {
  /// The heights' two scrolls, the first in `xy`, the second (read backwards) in `zw`.
  pub heights: Vec4,
  /// The four wave layers' runs and scroll, two layers a vector.
  pub wave_layers: [Vec4; 2],
  /// The rain ripples' three reads, then the wind layer's.
  pub map_offsets: [Vec4; 2],
  /// The caustics' two scrolls along `x`, the second read backwards, then the calm patches' drift.
  pub flow_scrolls: Vec4,
  /// The rain ripples' three layers' clocks, each from nought to one; `w` unused.
  pub ripple_clocks: Vec4,
}

impl EnhancedWaterScroll {
  /// The wave layers' turns, as their cosines and sines, and their tiling: the shader's `WAVE_TURN_COS`,
  /// `WAVE_TURN_SIN` and `WAVE_TILES`.
  pub(crate) const WAVE_TURN_COS: [f32; 4] = [1.0, 0.484810, -0.731354, -0.731354];
  pub(crate) const WAVE_TURN_SIN: [f32; 4] = [0.0, 0.874620, 0.681998, -0.681998];
  pub(crate) const WAVE_TILES: [f32; 4] = [1.0, std::f32::consts::GOLDEN_RATIO, 0.618034, 0.381966];
  /// The wind layer's tiling of the waves' coordinates, and the rain ripples' map's, then its second and third reads':
  /// the shader's `GUST_SCALE`, `RIPPLE_MAP_SCALE` and `RIPPLE_LAYER_SCALES`.
  pub(crate) const GUST_SCALE: f32 = 0.1;
  pub(crate) const RIPPLE_MAP_SCALE: f32 = 0.6;
  pub(crate) const RIPPLE_LAYER_SCALES: [f32; 2] = [0.61, 0.87];

  /// The waves' scroll across and down a second of their pace, and the map's repeats a wave-clock second a layer tiled
  /// once runs along its own axis.
  const WAVE_SCROLL: [f64; 2] = [0.065, 0.445];
  const WAVE_PACE: f64 = 0.3;
  /// The heights' two scrolls a second of their pace.
  const HEIGHT_SCROLLS: [f64; 4] = [0.065, 0.445, 0.105, 0.241];
  /// The caustics' two scrolls and the calm patches' drift a second of the flow.
  const FLOW_SCROLLS: [f64; 4] = [0.1, 0.07, 0.0021, 0.0013];
  /// How fast each of the rain ripples' three layers ripples, a second at the ripples' pace (`WaterFlow::ripples`).
  const RIPPLE_SPEEDS: [f64; 3] = [1.05, 1.31, 1.58];

  /// The reads at the `flow`'s sums; the waves' scroll moves none where the parallax is `is_marched`, as its march sets
  /// the waves' coordinates in its place.
  pub(crate) fn new(flow: &WaterFlow, is_marched: bool) -> Self {
    let wrap = |value: f64| LevelTime::wrap(value, 1.0);
    let scroll: [f64; 2] = if is_marched {
      [0.0; 2]
    } else {
      Self::WAVE_SCROLL.map(|rate| flow.waves * rate)
    };
    let layer = |index: usize| -> [f32; 2] {
      let cos: f64 = f64::from(Self::WAVE_TURN_COS[index]);
      let sin: f64 = f64::from(Self::WAVE_TURN_SIN[index]);
      let tile: f64 = f64::from(Self::WAVE_TILES[index]);
      let run: f64 = flow.waves * Self::WAVE_PACE / tile.sqrt();

      [
        wrap((cos * scroll[0] - sin * scroll[1]) * tile + run),
        wrap((sin * scroll[0] + cos * scroll[1]) * tile),
      ]
    };
    let scrolled = |scale: f64| scroll.map(|it| wrap(it * scale));
    let ripple_scale: f64 = f64::from(Self::RIPPLE_MAP_SCALE);
    let [second_scale, third_scale] = Self::RIPPLE_LAYER_SCALES.map(f64::from);
    let gust_scale: f64 = f64::from(Self::GUST_SCALE);
    let (first, second, third) = (
      scrolled(ripple_scale),
      scrolled(ripple_scale * second_scale),
      scrolled(ripple_scale * third_scale),
    );
    let gust: [f32; 2] = [0, 1].map(|axis| wrap((scroll[axis] + flow.gusts[axis]) * gust_scale));
    let [ripple_1, ripple_2, ripple_3] = Self::RIPPLE_SPEEDS.map(|speed| wrap(flow.ripples * speed));
    let [layer_0, layer_1, layer_2, layer_3] = [0, 1, 2, 3].map(layer);

    Self {
      heights: Vec4::from_array(Self::HEIGHT_SCROLLS.map(|rate| wrap(flow.heights * rate))),
      wave_layers: [
        Vec4::new(layer_0[0], layer_0[1], layer_1[0], layer_1[1]),
        Vec4::new(layer_2[0], layer_2[1], layer_3[0], layer_3[1]),
      ],
      map_offsets: [
        Vec4::new(first[0], first[1], second[0], second[1]),
        Vec4::new(third[0], third[1], gust[0], gust[1]),
      ],
      flow_scrolls: Vec4::from_array(Self::FLOW_SCROLLS.map(|rate| wrap(flow.seconds * rate))),
      ripple_clocks: Vec4::new(ripple_1, ripple_2, ripple_3, 0.0),
    }
  }
}
