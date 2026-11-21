use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_chunk::XRayByteOrder;
use xrf_thm::{
  ThmBumpChunk, ThmBumpMode, ThmDetailChunk, ThmFile, ThmFormat, ThmMaterial, ThmMaterialChunk, ThmMipFilter,
  ThmTextureEdit, ThmTextureFlag, ThmTextureFlags, ThmTextureParamChunk, ThmTextureType,
};
use xrf_utils::error_to_string;

/// The descriptor fields the editor owns, read off a `.thm` and written back onto one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureDescriptorForm {
  /// The thumbnail's texture type, which is the gate `LoadTHM` reads before anything else.
  pub texture_type: u32,
  pub bump_mode: u32,
  /// Bump texture path without extension, engine-style with backslashes. Empty when unused.
  pub bump_name: String,
  /// Detail texture path on the same terms.
  pub detail_name: String,
  pub detail_scale: f32,
  /// The whole `STextureParams` flag word, named bits and unnamed alike.
  pub flags: u32,
  pub format: u32,
  pub mip_filter: u32,
  pub border_color: u32,
  pub fade_color: u32,
  pub fade_amount: u32,
  /// Mip level the converter starts fading from.
  pub fade_delay: u8,
  pub material: u32,
  pub material_weight: f32,
  pub ext_normal_map_name: String,
  /// Read by the bump generator and by nothing at runtime.
  pub virtual_height: f32,
  /// What the descriptor claims its texture measures, which the DDS beside it is the authority on.
  pub width: u32,
  pub height: u32,
}

impl TextureDescriptorForm {
  /// The fields of a descriptor, or of a new one with the SDK's defaults when there is none.
  pub(crate) fn of(file: Option<&ThmFile>) -> Self {
    match file {
      Some(file) => Self::read(file),
      None => Self::read(&ThmFile::new_texture()),
    }
  }

  /// Reads the editable fields of a descriptor.
  pub(crate) fn read(file: &ThmFile) -> Self {
    let param: ThmTextureParamChunk = file.texture_param.unwrap_or_default();
    let detail: ThmDetailChunk = file.detail.clone().unwrap_or_default();
    let material: ThmMaterialChunk = file.material.unwrap_or_default();
    let bump: ThmBumpChunk = file.bump.clone().unwrap_or_default();

    Self {
      texture_type: file.texture_type.unwrap_or_default().into(),
      bump_mode: bump.mode.into(),
      bump_name: bump.name,
      detail_name: detail.name,
      detail_scale: detail.scale,
      flags: param.flags.raw(),
      format: param.format.into(),
      mip_filter: param.mip_filter.into(),
      border_color: param.border_color,
      fade_color: param.fade_color,
      fade_amount: param.fade_amount,
      fade_delay: file.fade_delay.unwrap_or_default(),
      material: material.material.into(),
      material_weight: material.weight,
      ext_normal_map_name: file.ext_normal_map_name.clone().unwrap_or_default(),
      virtual_height: bump.virtual_height,
      width: param.width,
      height: param.height,
    }
  }

  /// Whether the flag word says the texture's alpha means something, which tells `tfDXT1` from `tfADXT1`.
  pub(crate) fn has_alpha(&self) -> bool {
    ThmTextureFlags::from(self.flags).has(ThmTextureFlag::HasAlpha)
  }

  /// Writes these fields onto a descriptor, leaving everything else it carries alone.
  ///
  /// Only the fields that differ from what the descriptor reads as are set, so it gains a chunk it lacks only for a
  /// value other than the default that chunk reads as.
  pub(crate) fn apply_to(&self, file: &mut ThmFile) {
    self.edit_from(&Self::read(file)).apply_to(file);
  }

  /// The edit that takes a descriptor whose fields read as `current` to these.
  fn edit_from(&self, current: &Self) -> ThmTextureEdit {
    ThmTextureEdit {
      texture_type: Self::differing(&self.texture_type, &current.texture_type).map(ThmTextureType::from),
      format: Self::differing(&self.format, &current.format).map(ThmFormat::from),
      mip_filter: Self::differing(&self.mip_filter, &current.mip_filter).map(ThmMipFilter::from),
      width: Self::differing(&self.width, &current.width),
      height: Self::differing(&self.height, &current.height),
      set_flags: ThmTextureFlags::from(self.flags & !current.flags),
      cleared_flags: ThmTextureFlags::from(current.flags & !self.flags),
      border_color: Self::differing(&self.border_color, &current.border_color),
      fade_color: Self::differing(&self.fade_color, &current.fade_color),
      fade_amount: Self::differing(&self.fade_amount, &current.fade_amount),
      fade_delay: Self::differing(&self.fade_delay, &current.fade_delay),
      detail_name: Self::differing(&self.detail_name, &current.detail_name),
      detail_scale: Self::differing(&self.detail_scale, &current.detail_scale),
      material: Self::differing(&self.material, &current.material).map(ThmMaterial::from),
      material_weight: Self::differing(&self.material_weight, &current.material_weight),
      bump_mode: Self::differing(&self.bump_mode, &current.bump_mode).map(ThmBumpMode::from),
      bump_name: Self::differing(&self.bump_name, &current.bump_name),
      virtual_height: Self::differing(&self.virtual_height, &current.virtual_height),
      ext_normal_map_name: Self::differing(&self.ext_normal_map_name, &current.ext_normal_map_name),
    }
  }

  /// The descriptor this form describes, built onto `file` when there is one and onto a new one when there is not.
  pub(crate) fn to_descriptor(&self, file: Option<ThmFile>) -> ThmFile {
    let mut descriptor: ThmFile = file.unwrap_or_else(ThmFile::new_texture);

    self.apply_to(&mut descriptor);

    descriptor
  }

  /// The descriptor's bytes, built on `file` as [`Self::to_descriptor`] does, with `format` set when the texture
  /// written beside it has one the descriptor can name.
  ///
  /// # Errors
  ///
  /// Returns an error when the descriptor cannot be serialized.
  pub(crate) fn to_bytes(&self, file: Option<ThmFile>, format: Option<ThmFormat>) -> TauriResult<Vec<u8>> {
    let mut form: Self = self.clone();

    if let Some(format) = format {
      form.format = format.into();
    }

    form
      .to_descriptor(file)
      .write_to_bytes::<XRayByteOrder>()
      .map_err(error_to_string)
  }

  /// `value` where it differs from `current`, which is the only case an edit names it.
  fn differing<T: Clone + PartialEq>(value: &T, current: &T) -> Option<T> {
    (value != current).then(|| value.clone())
  }
}
