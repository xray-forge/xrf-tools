use image::{Rgba, RgbaImage};
use resvg::{tiny_skia, usvg};

use crate::core::types::TauriResult;

/// `svg` rasterised at `width` by `height` pixels, its view box stretched over them, its text set in the system's
/// fonts; transparent where it draws nothing, its alpha straight.
///
/// # Errors
///
/// Returns an error for an SVG that cannot be read, or a size of no pixels.
pub fn rasterise_annotations(svg: &str, (width, height): (u32, u32)) -> TauriResult<RgbaImage> {
  let mut options: usvg::Options<'_> = usvg::Options::default();

  options.fontdb_mut().load_system_fonts();

  let tree: usvg::Tree =
    usvg::Tree::from_str(svg, &options).map_err(|error| format!("Failed to read the annotations: {error}"))?;
  let mut pixmap: tiny_skia::Pixmap = tiny_skia::Pixmap::new(width, height)
    .ok_or_else(|| format!("Cannot draw the annotations at {width} by {height} pixels"))?;
  let size: tiny_skia::Size = tree.size();

  resvg::render(
    &tree,
    tiny_skia::Transform::from_scale(width as f32 / size.width(), height as f32 / size.height()),
    &mut pixmap.as_mut(),
  );

  let mut image: RgbaImage = RgbaImage::new(width, height);

  for (pixel, drawn) in image.pixels_mut().zip(pixmap.pixels()) {
    let colour: tiny_skia::ColorU8 = drawn.demultiply();

    *pixel = Rgba([colour.red(), colour.green(), colour.blue(), colour.alpha()]);
  }

  Ok(image)
}
