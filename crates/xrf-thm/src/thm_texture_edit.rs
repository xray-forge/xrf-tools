use xrf_error::XrfResult;

use crate::bump::thm_bump_mode::ThmBumpMode;
use crate::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::chunks::thm_detail_chunk::ThmDetailChunk;
use crate::chunks::thm_material_chunk::ThmMaterialChunk;
use crate::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use crate::texture::thm_format::ThmFormat;
use crate::texture::thm_material::ThmMaterial;
use crate::texture::thm_mip_filter::ThmMipFilter;
use crate::texture::thm_texture_flags::ThmTextureFlags;
use crate::texture::thm_texture_type::ThmTextureType;
use crate::thm_file::ThmFile;

/// The fields of a texture descriptor one edit sets, each left alone where the edit names no value for it.
///
/// Each field is set exactly as named: giving a descriptor a new type does not bring the flags, format and filter the
/// SDK's editor changes with it (`STextureParams::OnTypeChange`, see [`ThmFile::change_texture_type`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThmTextureEdit {
  pub texture_type: Option<ThmTextureType>,
  pub format: Option<ThmFormat>,
  pub mip_filter: Option<ThmMipFilter>,
  pub width: Option<u32>,
  pub height: Option<u32>,
  /// Bits of the flag word to set, named or not.
  pub set_flags: ThmTextureFlags,
  /// Bits of the flag word to clear; a bit in both masks ends up set.
  pub cleared_flags: ThmTextureFlags,
  pub border_color: Option<u32>,
  pub fade_color: Option<u32>,
  pub fade_amount: Option<u32>,
  pub fade_delay: Option<u8>,
  pub detail_name: Option<String>,
  pub detail_scale: Option<f32>,
  pub material: Option<ThmMaterial>,
  pub material_weight: Option<f32>,
  pub bump_mode: Option<ThmBumpMode>,
  pub bump_name: Option<String>,
  pub virtual_height: Option<f32>,
  pub ext_normal_map_name: Option<String>,
}

impl ThmTextureEdit {
  /// Sets the named fields on `file`, giving it a chunk it lacks, at the SDK's defaults, only where a field in that
  /// chunk is named.
  pub fn apply_to(&self, file: &mut ThmFile) {
    Self::assign(&mut file.texture_type, self.texture_type.map(Some));
    Self::assign(&mut file.fade_delay, self.fade_delay.map(Some));
    Self::assign(
      &mut file.ext_normal_map_name,
      self.ext_normal_map_name.clone().map(Some),
    );

    if self.is_editing_param() {
      self.apply_to_param(file.texture_param.get_or_insert_default());
    }

    if self.detail_name.is_some() || self.detail_scale.is_some() {
      let detail: &mut ThmDetailChunk = file.detail.get_or_insert_default();

      Self::assign(&mut detail.name, self.detail_name.clone());
      Self::assign(&mut detail.scale, self.detail_scale);
    }

    if self.material.is_some() || self.material_weight.is_some() {
      let material: &mut ThmMaterialChunk = file.material.get_or_insert_default();

      Self::assign(&mut material.material, self.material);
      Self::assign(&mut material.weight, self.material_weight);
    }

    if self.bump_mode.is_some() || self.bump_name.is_some() || self.virtual_height.is_some() {
      let bump: &mut ThmBumpChunk = file.bump.get_or_insert_default();

      Self::assign(&mut bump.mode, self.bump_mode);
      Self::assign(&mut bump.name, self.bump_name.clone());
      Self::assign(&mut bump.virtual_height, self.virtual_height);
    }
  }

  /// The descriptor `original` holds with the named fields set, as the bytes of a file, or `None` when the writer
  /// would not give `original` back byte for byte, so the fields the edit does not name could not be kept as they are.
  ///
  /// Every byte the edit does not change comes back as it was: equal bytes when the descriptor already holds every
  /// value named.
  ///
  /// # Errors
  ///
  /// Returns an error when `original` is not a descriptor the reader reads.
  pub fn rewrite(&self, original: &[u8]) -> XrfResult<Option<Vec<u8>>> {
    let mut file: ThmFile = ThmFile::read_from_bytes(original.to_vec())?;

    if file.write_to_bytes()? != original {
      return Ok(None);
    }

    self.apply_to(&mut file);

    Ok(Some(file.write_to_bytes()?))
  }

  /// Whether the edit names a field of the texture parameters.
  fn is_editing_param(&self) -> bool {
    self.format.is_some()
      || self.mip_filter.is_some()
      || self.width.is_some()
      || self.height.is_some()
      || self.set_flags.raw() != 0
      || self.cleared_flags.raw() != 0
      || self.border_color.is_some()
      || self.fade_color.is_some()
      || self.fade_amount.is_some()
  }

  fn apply_to_param(&self, param: &mut ThmTextureParamChunk) {
    Self::assign(&mut param.format, self.format);
    Self::assign(&mut param.mip_filter, self.mip_filter);
    Self::assign(&mut param.width, self.width);
    Self::assign(&mut param.height, self.height);
    Self::assign(&mut param.border_color, self.border_color);
    Self::assign(&mut param.fade_color, self.fade_color);
    Self::assign(&mut param.fade_amount, self.fade_amount);

    param.flags = ThmTextureFlags::from((param.flags.raw() & !self.cleared_flags.raw()) | self.set_flags.raw());
  }

  fn assign<T>(slot: &mut T, value: Option<T>) {
    if let Some(value) = value {
      *slot = value;
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use super::ThmTextureEdit;
  use crate::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
  use crate::tests::fixtures::{descriptor, raw_chunk};
  use crate::texture::thm_format::ThmFormat;
  use crate::texture::thm_texture_flag::ThmTextureFlag;
  use crate::texture::thm_texture_flags::ThmTextureFlags;
  use crate::thm_file::ThmFile;

  #[test]
  fn sets_only_the_fields_it_names() {
    let mut file: ThmFile = ThmFile::new_texture();
    let before: ThmTextureParamChunk = file.texture_param.expect("parameters");

    ThmTextureEdit {
      format: Some(ThmFormat::Dxt5),
      detail_name: Some(String::from("detail\\detail_grnd_earth")),
      set_flags: ThmTextureFlags::none().with(ThmTextureFlag::DiffuseDetail, true),
      cleared_flags: ThmTextureFlags::none().with(ThmTextureFlag::DitherColor, true),
      ..ThmTextureEdit::default()
    }
    .apply_to(&mut file);

    let after: ThmTextureParamChunk = file.texture_param.expect("parameters");

    assert_eq!(after.format, ThmFormat::Dxt5);
    assert!(after.flags.has(ThmTextureFlag::DiffuseDetail));
    assert!(!after.flags.has(ThmTextureFlag::DitherColor));
    assert_eq!(after.mip_filter, before.mip_filter);
    assert_eq!(file.detail.as_ref().map(|detail| detail.scale), Some(1.0));
    assert_eq!(file.bump, ThmFile::new_texture().bump);
  }

  #[test]
  fn sets_and_clears_bits_the_sdk_has_no_name_for() {
    let mut file: ThmFile = ThmFile::new_texture();

    ThmTextureEdit {
      set_flags: ThmTextureFlags::from(1 << 30),
      cleared_flags: ThmTextureFlags::from(1 << 0),
      ..ThmTextureEdit::default()
    }
    .apply_to(&mut file);

    let flags: ThmTextureFlags = file.texture_param.expect("parameters").flags;

    assert_eq!(flags.unnamed(), 1 << 30);
    assert!(!flags.has(ThmTextureFlag::GenerateMipMaps));
  }

  #[test]
  fn gives_a_descriptor_a_chunk_it_lacks_only_where_a_field_in_it_is_named() {
    let mut file: ThmFile = ThmFile::default();

    ThmTextureEdit {
      bump_name: Some(String::from("wpn\\wpn_pm_bump")),
      ..ThmTextureEdit::default()
    }
    .apply_to(&mut file);

    assert_eq!(file.bump.map(|bump| bump.name), Some(String::from("wpn\\wpn_pm_bump")));
    assert_eq!(file.texture_param, None);
    assert_eq!(file.detail, None);
  }

  #[test]
  fn rewrites_only_the_bytes_of_the_field_named() -> XrfResult {
    let original: Vec<u8> = descriptor();
    let edited: Vec<u8> = ThmTextureEdit {
      fade_amount: Some(40),
      ..ThmTextureEdit::default()
    }
    .rewrite(&original)?
    .expect("a descriptor the writer gives back");
    let differing: Vec<usize> = (0..original.len())
      .filter(|index| original[*index] != edited[*index])
      .collect();

    assert_eq!(edited.len(), original.len());
    assert_eq!(differing.len(), 1, "the amount's low byte, 0 to 40: {differing:?}");
    assert_eq!(edited[differing[0]], 40);
    assert_eq!(
      ThmTextureEdit::default().rewrite(&original)?,
      Some(original),
      "naming nothing gives the file back"
    );

    Ok(())
  }

  #[test]
  fn gives_nothing_for_a_descriptor_the_writer_would_not_give_back() -> XrfResult {
    let original: Vec<u8> = descriptor();
    // An unknown chunk after the ten-byte version chunk, which the writer would move to the end of the file.
    let reordered: Vec<u8> = [&original[..10], &raw_chunk(0x0900, &[1, 2, 3]), &original[10..]].concat();

    assert_eq!(ThmTextureEdit::default().rewrite(&reordered)?, None);

    Ok(())
  }
}
