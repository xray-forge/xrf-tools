use std::sync::{Arc, OnceLock};

use image::{Rgba, RgbaImage};
use resvg::usvg::fontdb::Database;
use resvg::{tiny_skia, usvg};

use crate::core::types::TauriResult;

/// The system's fonts, read once a run: reading them takes a few hundred milliseconds.
static SYSTEM_FONTS: OnceLock<Arc<Database>> = OnceLock::new();

/// `svg` rasterised at `width` by `height` pixels, its view box stretched over them, its text set in the system's
/// fonts; transparent where it draws nothing, its alpha straight.
///
/// # Errors
///
/// Returns an error for an SVG that cannot be read, or a size of no pixels.
pub fn rasterise_annotations(svg: &str, (width, height): (u32, u32)) -> TauriResult<RgbaImage> {
  let options: usvg::Options<'_> = usvg::Options {
    fontdb: Arc::clone(SYSTEM_FONTS.get_or_init(|| {
      let mut fonts: Database = Database::new();

      fonts.load_system_fonts();

      Arc::new(fonts)
    })),
    ..usvg::Options::default()
  };
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
