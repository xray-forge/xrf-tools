use image::RgbaImage;
use xrf_error::{XrfError, XrfResult};
use xrf_thm::{ThmFile, ThmFormat, ThmTextureType};

/// The external normal map a bump descriptor names, as its caller found it: the pixels of its game texture and its own
/// descriptor, which `CImageManager::MakeGameTexture` checks before it lets the map replace the derived normals.
pub struct BuildTextureNormalMap {
  pub descriptor: ThmFile,
  pub image: RgbaImage,
}

impl BuildTextureNormalMap {
  /// The pixels, once the map is one the SDK accepts for the bump named `name` built from a source of `size`.
  ///
  /// # Errors
  ///
  /// Returns an error, as the SDK does, when the map's descriptor is not a normal map, its format is not 32-bit RGBA, or
  /// its size is not the source's.
  pub fn accept(&self, name: &str, size: (u32, u32)) -> XrfResult<&RgbaImage> {
    if self.descriptor.texture_type.unwrap_or_default() != ThmTextureType::NormalMap {
      return Err(XrfError::new_texture_processing_error(format!(
        "External normal map '{name}' is not described as a normal map, the only type the SDK takes for one"
      )));
    }

    if self.descriptor.texture_param.as_ref().map(|param| param.format) != Some(ThmFormat::Rgba) {
      return Err(XrfError::new_texture_processing_error(format!(
        "External normal map '{name}' is not described as 32-bit RGBA, the only format the SDK takes for one"
      )));
    }

    if self.image.dimensions() != size {
      return Err(XrfError::new_texture_processing_error(format!(
        "External normal map '{name}' is {}x{} and the bump source {}x{}, which the SDK refuses",
        self.image.width(),
        self.image.height(),
        size.0,
        size.1
      )));
    }

    Ok(&self.image)
  }
}
