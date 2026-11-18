use glam::Vec3;
use xrf_dds::RgbaImage;
use xrf_math::Vector2d;

use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::atlas::lightmap_image_pair::LightmapImagePair;
use crate::light::lightmap::atlas::lightmap_images::LightmapImages;
use crate::light::point::light_color::LightColor;
use crate::light::vertex::light_tree_color::LightTreeColor;

/// The light a level compiled without the light step carries, neutral constants: an open sky for the sun, which the
/// renderers shadow in real time, a middle hemisphere, and no static light, which they light in real time too. Every
/// lightmapped visual samples the middle of one small pair of it, and every implicitly lit texture has maps of it.
pub(crate) struct LightDraft;

impl LightDraft {
  /// A static vertex's light.
  pub(crate) const VERTEX: LightColor = LightColor {
    rgb: Vec3::ZERO,
    hemi: Self::HEMISPHERE,
    sun: Self::SUN,
  };

  /// A tree vertex's light.
  pub(crate) const TREE_VERTEX: LightColor = LightColor {
    rgb: Vec3::ZERO,
    hemi: Self::TREE_HEMISPHERE,
    sun: Self::TREE_SUN,
  };

  /// The sun term, of a vertex colour's fourth byte and a lightmap's.
  pub(crate) const SUN: f32 = 1.0;

  /// The hemisphere term, of a normal's fourth byte and a lightmap's.
  pub(crate) const HEMISPHERE: f32 = 0.5;

  /// The page of the one lightmap pair a lightmapped visual is dressed with where no charts are cut, the first
  /// ([`crate::LightmapFaces::get_textures`] names it).
  pub(crate) const LIGHTMAP_PAGE: u32 = 0;

  /// The lightmap's side in texels: one uniform colour needs no more.
  pub(crate) const LIGHTMAP_SIDE: u32 = 4;

  /// Where every lightmapped vertex samples it, its middle.
  pub(crate) const LIGHTMAP_COORDINATE: Vector2d = Vector2d::new(0.5, 0.5);

  /// The one neutral lightmap pair, [`Self::LIGHTMAP_SIDE`] texels a side: `_1` no static light and the sun in alpha,
  /// `_2` the sun in every colour and the hemisphere in alpha.
  pub(crate) fn get_lightmaps() -> LightmapImages {
    let sun: u8 = LightColor::get_byte(Self::SUN);
    let texels: usize = (Self::LIGHTMAP_SIDE * Self::LIGHTMAP_SIDE) as usize;

    LightmapImages {
      side: Self::LIGHTMAP_SIDE,
      pages: vec![LightmapImagePair {
        first: vec![[0, 0, 0, sun]; texels],
        second: vec![[sun, sun, sun, LightColor::get_byte(Self::HEMISPHERE)]; texels],
      }],
    }
  }

  /// The neutral maps of an implicitly lit texture named `name` whose image is `image`: the image with the middle
  /// hemisphere in alpha, and a map of no static light and the open sun, the size of the image.
  pub(crate) fn get_implicit_map(name: String, image: &RgbaImage) -> ImplicitMap {
    let (width, height) = image.dimensions();
    let hemisphere: u8 = LightColor::get_byte(Self::HEMISPHERE);
    let base: Vec<[u8; 4]> = image
      .pixels()
      .map(|pixel| [pixel.0[0], pixel.0[1], pixel.0[2], hemisphere])
      .collect();
    let lightmap: Vec<[u8; 4]> = vec![[0, 0, 0, LightColor::get_byte(Self::SUN)]; base.len()];

    ImplicitMap {
      name,
      width,
      height,
      base,
      lightmap,
    }
  }

  /// A tree's hemisphere term, as shipped trees carry it in effect.
  pub(crate) const TREE_HEMISPHERE: f32 = 0.67;

  /// A tree's sun term, likewise.
  pub(crate) const TREE_SUN: f32 = 0.38;

  /// A tree placement's scale, `c_scale`: none of a vertex's own light kept.
  pub(crate) const TREE_SCALE: LightTreeColor = LightTreeColor {
    rgb: Vec3::ZERO,
    hemi: 0.0,
    sun: 0.0,
  };

  /// A tree placement's bias, `c_bias`: no static light, and the tree's hemisphere and sun.
  pub(crate) const TREE_BIAS: LightTreeColor = LightTreeColor {
    rgb: Vec3::ZERO,
    hemi: Self::TREE_HEMISPHERE,
    sun: Self::TREE_SUN,
  };
}
