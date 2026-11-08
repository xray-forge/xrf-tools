use image::RgbaImage;
use xrf_error::{XrfError, XrfResult};

use crate::mip::dds_mip_chain::DdsMipChain;

/// The levels of one surface as the encoder writes them, borrowed from the chains that hold them: every layer's levels
/// in turn, base first, each layer alike in the size of every level.
pub(crate) struct DdsSurfaceLevels<'a> {
  /// Layers the surface holds.
  layers: usize,
  levels: Vec<&'a RgbaImage>,
}

impl<'a> DdsSurfaceLevels<'a> {
  /// One chain as a surface of one layer.
  ///
  /// # Errors
  ///
  /// Returns an error when the chain holds no level, which no chain built here does.
  pub(crate) fn of_chain(chain: &'a DdsMipChain) -> XrfResult<Self> {
    Self::of_layers(std::slice::from_ref(chain))
  }

  /// Six chains as a cube, each face's levels in turn, `+x -x +y -y +z -z`.
  ///
  /// # Errors
  ///
  /// Returns an error unless there are six faces, square and alike in the size of every level.
  pub(crate) fn of_cube(faces: &'a [DdsMipChain]) -> XrfResult<Self> {
    let surface: Self = Self::of_layers(faces)?;
    let (width, height): (u32, u32) = surface.base().dimensions();

    if surface.layers != 6 || width != height {
      return Err(XrfError::new_texture_processing_error(format!(
        "A cube is six square faces, not {} of {width} by {height}",
        surface.layers
      )));
    }

    Ok(surface)
  }

  /// Chains alike in the size of every level as one surface, a layer each.
  ///
  /// # Errors
  ///
  /// Returns an error when there is no layer or level, or the layers differ in the size of a level.
  fn of_layers(layers: &'a [DdsMipChain]) -> XrfResult<Self> {
    let [first, ..] = layers else {
      return Err(XrfError::new_texture_processing_error(
        "Cannot encode a surface holding no layers",
      ));
    };

    if first.levels().is_empty() {
      return Err(XrfError::new_texture_processing_error(
        "Cannot encode a mip chain holding no levels",
      ));
    }

    let is_alike = |layer: &DdsMipChain| {
      layer
        .levels()
        .iter()
        .map(RgbaImage::dimensions)
        .eq(first.levels().iter().map(RgbaImage::dimensions))
    };

    if !layers.iter().all(is_alike) {
      return Err(XrfError::new_texture_processing_error(
        "Layers of one surface must share the size of every level",
      ));
    }

    Ok(Self {
      layers: layers.len(),
      levels: layers.iter().flat_map(DdsMipChain::levels).collect(),
    })
  }

  /// The first layer's base level, whose size is the surface's.
  pub(crate) fn base(&self) -> &'a RgbaImage {
    self.levels[0]
  }

  /// Layers the surface holds.
  pub(crate) fn layers(&self) -> usize {
    self.layers
  }

  /// Levels each layer holds, its base included.
  pub(crate) fn mipmaps(&self) -> usize {
    self.levels.len() / self.layers
  }

  /// Every layer's levels in turn, base first.
  pub(crate) fn levels(&self) -> &[&'a RgbaImage] {
    &self.levels
  }
}

#[cfg(test)]
mod tests {
  use image::RgbaImage;
  use xrf_error::XrfResult;

  use super::DdsSurfaceLevels;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  fn chain(width: u32, height: u32) -> XrfResult<DdsMipChain> {
    DdsMipChain::build(&RgbaImage::new(width, height), DdsMipmaps::Filtered(DdsMipFilter::Box))
  }

  #[test]
  fn lays_a_chain_out_as_one_layer_base_first() -> XrfResult {
    let chain: DdsMipChain = chain(4, 4)?;
    let surface: DdsSurfaceLevels<'_> = DdsSurfaceLevels::of_chain(&chain)?;

    assert_eq!((surface.layers(), surface.mipmaps()), (1, 3));
    assert_eq!(surface.base().dimensions(), (4, 4));
    assert_eq!(
      surface
        .levels()
        .iter()
        .map(|level| level.dimensions())
        .collect::<Vec<(u32, u32)>>(),
      vec![(4, 4), (2, 2), (1, 1)]
    );

    Ok(())
  }

  #[test]
  fn lays_six_faces_out_as_one_cube_and_refuses_anything_else() -> XrfResult {
    let faces: Vec<DdsMipChain> = (0..6).map(|_| chain(4, 4)).collect::<XrfResult<_>>()?;
    let cube: DdsSurfaceLevels<'_> = DdsSurfaceLevels::of_cube(&faces)?;

    assert_eq!((cube.layers(), cube.mipmaps(), cube.levels().len()), (6, 3, 18));
    assert!(DdsSurfaceLevels::of_cube(&faces[..5]).is_err());
    assert!(DdsSurfaceLevels::of_cube(&[]).is_err());

    let oblong: Vec<DdsMipChain> = (0..6).map(|_| chain(8, 4)).collect::<XrfResult<_>>()?;
    // Six squares, the last twice the size of the others.
    let uneven: Vec<DdsMipChain> = [4, 4, 4, 4, 4, 8]
      .into_iter()
      .map(|side| chain(side, side))
      .collect::<XrfResult<_>>()?;

    assert!(DdsSurfaceLevels::of_cube(&oblong).is_err());
    assert!(DdsSurfaceLevels::of_cube(&uneven).is_err());

    Ok(())
  }
}
