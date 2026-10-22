use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use xrf_error::{XrfError, XrfResult};

use crate::capture::cube_face::CubeFace;
use crate::contract::render_capture::RenderCapture;

/// Each face's place in the horizontal cross, in faces across and down, in the engine's order `+x -x +y -y +z -z`: the
/// up face above the forward one, then the left, forward, right and back faces in a row, the down face below, so every
/// edge meets the face it touches in the cube.
const PLACES: [(u32, u32); CubeFace::COUNT] = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)];

/// A cubemap put together from its faces as each comes back, reduced by Lanczos, as a horizontal cross four faces
/// across and three down, transparent where no face lies.
pub struct CubeAssembly {
  size: u32,
  cross: RgbaImage,
}

impl CubeAssembly {
  /// An empty cross of faces `size` pixels square.
  pub fn new(size: u32) -> Self {
    Self {
      size,
      cross: RgbaImage::new(size * 4, size * 3),
    }
  }

  /// Puts in a face from what was drawn of it, opaque.
  ///
  /// # Errors
  ///
  /// Returns an error for a face of another size than the cross's, or one drawn at another than its own.
  pub fn place(&mut self, face: &CubeFace, drawn: RenderCapture) -> XrfResult {
    let (side, _): (u32, u32) = face.get_drawn_size();

    if face.size != self.size {
      return Err(XrfError::new_invalid_error(format!(
        "A cubemap of {} pixel faces was handed a face of {}",
        self.size, face.size
      )));
    }

    if (drawn.width, drawn.height) != (side, side) {
      return Err(XrfError::new_invalid_error(format!(
        "A cubemap's face came back {} by {} pixels, not the {side} square it was drawn at",
        drawn.width, drawn.height
      )));
    }

    let drawn: RgbaImage = RgbaImage::from_raw(side, side, drawn.pixels)
      .ok_or_else(|| XrfError::new_invalid_error(format!("A cubemap's face's pixels do not fill {side} square")))?;
    let mut reduced: RgbaImage = if face.supersampling > 1 {
      imageops::resize(&drawn, face.size, face.size, FilterType::Lanczos3)
    } else {
      drawn
    };

    for pixel in reduced.pixels_mut() {
      pixel.0[3] = u8::MAX;
    }

    let (x, y): (u32, u32) = PLACES[face.index % CubeFace::COUNT];

    imageops::replace(
      &mut self.cross,
      &reduced,
      i64::from(x * self.size),
      i64::from(y * self.size),
    );

    Ok(())
  }

  /// The cross, with every face put in.
  pub fn finish(self) -> RgbaImage {
    self.cross
  }

  /// The six faces of a cross, in the engine's order `+x -x +y -y +z -z`.
  ///
  /// # Errors
  ///
  /// Returns an error for an image that is not four faces across and three down.
  pub fn split(cross: &RgbaImage) -> XrfResult<Vec<RgbaImage>> {
    let size: u32 = cross.width() / 4;

    if size == 0 || cross.width() != size * 4 || cross.height() != size * 3 {
      return Err(XrfError::new_invalid_error(format!(
        "A cubemap's cross is four faces across and three down, not {} by {} pixels",
        cross.width(),
        cross.height()
      )));
    }

    Ok(
      PLACES
        .iter()
        .map(|(x, y)| imageops::crop_imm(cross, x * size, y * size, size, size).to_image())
        .collect(),
    )
  }
}
