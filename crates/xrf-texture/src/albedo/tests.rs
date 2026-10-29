use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_error::XrfResult;

use crate::albedo::{AlbedoVerdict, TextureAlbedo};

/// The linear value the renderer lights an encoded byte as.
fn linear(value: u8) -> f32 {
  xrf_math::srgb_decoded(f32::from(value) / 255.0)
}

/// A flat grey texture `side` texels a side, its chain as `mipmaps` builds it.
fn grey_texture(side: u32, mipmaps: DdsMipmaps) -> XrfResult<DdsFile> {
  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast).encode(&DdsMipChain::build(
    &RgbaImage::from_pixel(side, side, grey(128, 255)),
    mipmaps,
  )?)
}

fn grey(value: u8, alpha: u8) -> Rgba<u8> {
  Rgba([value, value, value, alpha])
}

#[test]
fn a_flat_grey_is_measured_as_its_decoded_value() {
  let measured: TextureAlbedo =
    TextureAlbedo::measure(&RgbaImage::from_pixel(16, 16, grey(128, 255)), None).expect("texels to measure");

  assert_eq!(measured.texels, 256);
  assert!((measured.median - linear(128)).abs() < 1e-4, "{}", measured.median);
  assert!((measured.mean[0] - linear(128)).abs() < 1e-4, "{:?}", measured.mean);
  assert_eq!(measured.coarse_spread, 0.0, "nothing painted, nothing spread");
  assert_eq!(measured.verdict, AlbedoVerdict::Within);
}

#[test]
fn a_file_is_read_at_its_largest_level_within_the_measured_side() -> XrfResult {
  let chained: TextureAlbedo =
    TextureAlbedo::read(&grey_texture(1024, DdsMipmaps::Filtered(DdsMipFilter::Box))?, None)?.expect("chained");
  let single: TextureAlbedo = TextureAlbedo::read(&grey_texture(1024, DdsMipmaps::Disabled)?, None)?.expect("single");

  assert_eq!((chained.width, chained.height), (512, 512));
  assert_eq!(
    (single.width, single.height),
    (1024, 1024),
    "a file without a chain is read at its base"
  );

  Ok(())
}

#[test]
fn colours_past_the_range_are_judged_by_their_median() {
  let dark: TextureAlbedo = TextureAlbedo::measure(&RgbaImage::from_pixel(8, 8, grey(20, 255)), None).expect("dark");
  let bright: TextureAlbedo =
    TextureAlbedo::measure(&RgbaImage::from_pixel(8, 8, grey(252, 255)), None).expect("bright");

  assert_eq!((dark.verdict, dark.dark_share), (AlbedoVerdict::Dark, 1.0));
  assert_eq!((bright.verdict, bright.bright_share), (AlbedoVerdict::Bright, 1.0));
}

#[test]
fn a_cut_out_is_measured_over_the_texels_its_alpha_test_keeps() {
  let image: RgbaImage = RgbaImage::from_fn(16, 16, |x, _| if x < 8 { grey(0, 0) } else { grey(160, 255) });
  let opaque: TextureAlbedo = TextureAlbedo::measure(&image, None).expect("every texel");
  let tested: TextureAlbedo = TextureAlbedo::measure(&image, Some(200)).expect("the kept half");

  assert_eq!(opaque.texels, 256);
  assert_eq!(tested.texels, 128);
  assert!(
    (tested.median - linear(160)).abs() < 1e-4,
    "the killed black is not measured"
  );
  assert_eq!(tested.dark_share, 0.0);
  assert!(
    TextureAlbedo::measure(&RgbaImage::from_pixel(4, 4, grey(160, 10)), Some(200)).is_none(),
    "nothing kept, nothing measured"
  );
}

#[test]
fn large_scale_shade_widens_the_coarse_spread() {
  let lit_and_shaded: RgbaImage =
    RgbaImage::from_fn(64, 64, |_, y| if y < 32 { grey(200, 255) } else { grey(60, 255) });
  // Fine grain of the same two values, every cell holding both alike.
  let grained: RgbaImage = RgbaImage::from_fn(64, 64, |x, y| {
    if (x + y) % 2 == 0 {
      grey(200, 255)
    } else {
      grey(60, 255)
    }
  });

  let spread: f32 = TextureAlbedo::measure(&lit_and_shaded, None)
    .expect("shaded")
    .coarse_spread;

  assert!((spread - (linear(200) / linear(60)).log2()).abs() < 1e-3, "{spread}");
  assert!(TextureAlbedo::measure(&grained, None).expect("grained").coarse_spread < 1e-3);
}
