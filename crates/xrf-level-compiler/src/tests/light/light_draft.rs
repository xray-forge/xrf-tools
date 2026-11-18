//! The neutral light a level compiled without the light step carries.

use xrf_dds::RgbaImage;

use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::light_draft::LightDraft;
use crate::light::point::light_color::LightColor;

/// An implicitly lit texture left unlit keeps its colours under the middle hemisphere, and its `_lm` holds no static
/// light and the open sun, so terrain draws lit by the renderer alone.
#[test]
fn gives_an_unlit_implicit_texture_neutral_maps_the_size_of_its_image() {
  let image: RgbaImage = RgbaImage::from_raw(2, 1, vec![10, 20, 30, 255, 40, 50, 60, 0]).expect("two texels");
  let map: ImplicitMap = LightDraft::get_implicit_map(String::from("terrain\\terrain_test"), &image);
  let hemisphere: u8 = LightColor::get_byte(LightDraft::HEMISPHERE);
  let sun: u8 = LightColor::get_byte(LightDraft::SUN);

  assert_eq!((map.width, map.height), (2, 1));
  assert_eq!(map.base, vec![[10, 20, 30, hemisphere], [40, 50, 60, hemisphere]]);
  assert_eq!(map.lightmap, vec![[0, 0, 0, sun]; 2]);
}
