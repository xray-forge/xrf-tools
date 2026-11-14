use glam::Vec3;
use xrf_math::Vector2d;

use crate::light::light_color::LightColor;

/// The light a level compiled without the light step carries, neutral constants: an open sky for the sun, which the
/// renderers shadow in real time, a middle hemisphere, and no static light, which they light in real time too. Every
/// lightmapped visual samples the middle of one small pair of it.
pub struct GeometryDraftLight;

impl GeometryDraftLight {
  /// A static vertex's light.
  pub const VERTEX: LightColor = LightColor {
    rgb: Vec3::ZERO,
    hemi: Self::HEMISPHERE,
    sun: Self::SUN,
  };

  /// A tree vertex's light.
  pub const TREE_VERTEX: LightColor = LightColor {
    rgb: Vec3::ZERO,
    hemi: Self::TREE_HEMISPHERE,
    sun: Self::TREE_SUN,
  };

  /// The sun term, of a vertex colour's fourth byte and a lightmap's.
  pub const SUN: f32 = 1.0;

  /// The hemisphere term, of a normal's fourth byte and a lightmap's.
  pub const HEMISPHERE: f32 = 0.5;

  /// The one lightmap pair a lightmapped visual is dressed with where no charts are cut.
  pub const LIGHTMAP_TEXTURES: [&'static str; 2] = ["lmap#1_1", "lmap#1_2"];

  /// The lightmap's side in texels: one uniform colour needs no more.
  pub const LIGHTMAP_SIDE: u32 = 4;

  /// Where every lightmapped vertex samples it, its middle.
  pub const LIGHTMAP_COORDINATE: Vector2d = Vector2d::new(0.5, 0.5);

  /// The neutral lightmap pair, [`Self::LIGHTMAP_SIDE`] texels a side: `_1` no static light and the sun in alpha, `_2`
  /// the sun in every colour and the hemisphere in alpha (`lm_layer::Pack`, `Pack_hemi`).
  pub fn get_lightmap_page() -> [Vec<[u8; 4]>; 2] {
    let sun: u8 = LightColor::get_byte(Self::SUN);
    let texels: usize = (Self::LIGHTMAP_SIDE * Self::LIGHTMAP_SIDE) as usize;

    [
      vec![[0, 0, 0, sun]; texels],
      vec![[sun, sun, sun, LightColor::get_byte(Self::HEMISPHERE)]; texels],
    ]
  }

  /// A tree's hemisphere term, as shipped trees carry it in effect.
  pub const TREE_HEMISPHERE: f32 = 0.67;

  /// A tree's sun term, likewise.
  pub const TREE_SUN: f32 = 0.38;

  /// A tree placement's scale, `c_scale`: red, green, blue, hemisphere and sun, none of a vertex's own light kept.
  pub const TREE_SCALE: [f32; 5] = [0.0; 5];

  /// A tree placement's bias, `c_bias`: no static light, and the tree's hemisphere and sun.
  pub const TREE_BIAS: [f32; 5] = [0.0, 0.0, 0.0, Self::TREE_HEMISPHERE, Self::TREE_SUN];
}
