use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_app_core::jobs::JobResource;
use xrf_dds::DdsMipFilter;
use xrf_job::JobHandle;
use xrf_texture::{
  GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, ImageFileReader, TextureEncodingQuality,
};
use xrf_utils::error_to_string;

/// What a bump pair generation was asked to do.
///
/// The height source is required and everything else refines it, so a caller with only a height map still gets a
/// pair. Gloss is a plane of its own here, where the SDK's generator reads it from its one source's alpha.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesMakeBumpRequest {
  /// Path of the texture the pair belongs to, without the `_bump` suffix or an extension.
  pub destination: String,
  /// Path of the image the relief is read from, averaged across its colour channels.
  pub height: String,
  /// Path of a gloss mask, averaged the same way.
  pub gloss: Option<String>,
  pub gloss_constant: f32,
  /// Path of a normal map to use instead of deriving one from the height, of the same size.
  pub normal_map: Option<String>,
  /// `bump_virtual_height` of the descriptor, read here and nowhere at runtime; the SDK's default when absent.
  pub virtual_height: Option<f32>,
  /// Kernel the pair's chain is reduced with, by its SDK name.
  pub mip_filter: Option<String>,
  pub quality: TextureEncodingQuality,
}

impl TexturesMakeBumpRequest {
  /// The kernel the pair's chain is reduced with, a box where none is named.
  ///
  /// # Errors
  ///
  /// Returns an error naming the kernel when it is not one of the fourteen.
  pub fn to_mip_filter(&self) -> TauriResult<DdsMipFilter> {
    match self.mip_filter.as_deref() {
      None => Ok(DdsMipFilter::Box),
      Some(label) => DdsMipFilter::from_label(label).ok_or_else(|| format!("Unexpected mip filter '{label}'")),
    }
  }

  /// Both files the pair is written as, following the generator's naming rule without decoding any image, as write
  /// claims.
  pub fn to_resources(&self) -> Vec<JobResource> {
    GenerateBumpProcessor::pair_paths(Path::new(&self.destination))
      .iter()
      .map(JobResource::file)
      .collect()
  }

  /// The generation `job` runs: every image the request names read, and the gloss it asks for settled.
  ///
  /// # Errors
  ///
  /// Returns an error when an image cannot be read or decoded.
  pub fn to_options(&self, job: JobHandle, mip_filter: DdsMipFilter) -> TauriResult<GenerateBumpOptions> {
    Ok(GenerateBumpOptions {
      job,
      destination: PathBuf::from(&self.destination),
      height: ImageFileReader::read_rgba(&self.height).map_err(error_to_string)?,
      gloss: match &self.gloss {
        Some(path) => GenerateBumpGloss::Mask(ImageFileReader::read_rgba(path).map_err(error_to_string)?),
        None => GenerateBumpGloss::Constant(self.gloss_constant),
      },
      normal_map: match &self.normal_map {
        Some(path) => Some(ImageFileReader::read_rgba(path).map_err(error_to_string)?),
        None => None,
      },
      virtual_height: self
        .virtual_height
        .unwrap_or(GenerateBumpOptions::DEFAULT_VIRTUAL_HEIGHT),
      mip_filter,
      quality: self.quality.to_quality(),
    })
  }
}
