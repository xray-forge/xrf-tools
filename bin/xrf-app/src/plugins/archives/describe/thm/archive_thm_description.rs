use serde::Serialize;
use xrf_error::XrfResult;
use xrf_spawn::XRayByteOrder;
use xrf_thm::{ThmFile, ThmTextureParamChunk};
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;
use crate::plugins::archives::describe::thm::archive_thm_bump::ArchiveThmBump;
use crate::plugins::archives::describe::thm::archive_thm_detail::ArchiveThmDetail;
use crate::plugins::archives::describe::thm::archive_thm_file::ArchiveThmFile;
use crate::plugins::archives::describe::thm::archive_thm_material::ArchiveThmMaterial;
use crate::plugins::archives::describe::thm::archive_thm_parameters::ArchiveThmParameters;
use crate::plugins::archives::describe::thm::archive_thm_texture::ArchiveThmTexture;
use crate::plugins::archives::describe::thm::archive_thm_texture_type::ArchiveThmTextureType;

/// Everything the viewer says about one texture descriptor.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDescription {
  pub texture: ArchiveThmTexture,
  pub texture_type: ArchiveThmTextureType,
  pub bump: Option<ArchiveThmBump>,
  pub detail: Option<ArchiveThmDetail>,
  /// The normal map replacing the one the generator would derive, when the file names one.
  pub external_normal_map: Option<ArchiveReference>,
  pub material: Option<ArchiveThmMaterial>,
  pub parameters: Option<ArchiveThmParameters>,
  /// Mip level the converter starts fading from, `fade_delay` (`ETextureParams.h:88`).
  pub fade_delay: Option<u8>,
  pub file: ArchiveThmFile,
}

impl ArchiveThmDescription {
  /// Reads the descriptor an entry holds and resolves what it names against the subject being browsed.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a descriptor this reader can walk.
  /// A chunk it does not recognise is not a failure: [`ThmFile`] keeps it, and it is reported as an extra chunk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: ThmFile = ThmFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;
    let parameters: Option<&ThmTextureParamChunk> = file.texture_param.as_ref();

    Ok(Self {
      texture: ArchiveThmTexture::of(source, name, file.texture_type(), parameters),
      texture_type: ArchiveThmTextureType::of(&file),
      bump: file.bump.as_ref().map(|bump| ArchiveThmBump::of(source, bump)),
      detail: file
        .detail
        .as_ref()
        .map(|detail| ArchiveThmDetail::of(source, detail, file.used_detail_usage())),
      external_normal_map: file
        .ext_normal_map_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .map(|name| ArchiveReference::resolve(source, XrayAssetType::Dds, name)),
      material: file.material.as_ref().map(ArchiveThmMaterial::of),
      parameters: parameters.map(ArchiveThmParameters::of),
      fade_delay: file.fade_delay,
      file: ArchiveThmFile::of(&file),
    })
  }
}
