use serde::{Deserialize, Serialize};
use xrf_thm::{
  ThmBumpChunk, ThmBumpMode, ThmDetailChunk, ThmFile, ThmFormat, ThmMaterial, ThmMaterialChunk, ThmMipFilter,
  ThmTextureFlags, ThmTextureParamChunk, ThmTextureType,
};

/// The descriptor fields the editor owns, read off a `.thm` and written back onto one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureDescriptorForm {
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
  /// Reads the editable fields of a descriptor.
  pub fn read(file: &ThmFile) -> Self {
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

  /// Writes these fields onto a descriptor, leaving everything else it carries alone.
  pub fn apply_to(&self, file: &mut ThmFile) {
    apply_chunk(&mut file.texture_type, ThmTextureType::from(self.texture_type));
    apply_chunk(
      &mut file.texture_param,
      ThmTextureParamChunk {
        format: ThmFormat::from(self.format),
        flags: ThmTextureFlags::from(self.flags),
        border_color: self.border_color,
        fade_color: self.fade_color,
        fade_amount: self.fade_amount,
        mip_filter: ThmMipFilter::from(self.mip_filter),
        width: self.width,
        height: self.height,
      },
    );
    apply_chunk(
      &mut file.detail,
      ThmDetailChunk {
        name: self.detail_name.clone(),
        scale: self.detail_scale,
      },
    );
    apply_chunk(
      &mut file.material,
      ThmMaterialChunk {
        material: ThmMaterial::from(self.material),
        weight: self.material_weight,
      },
    );
    apply_chunk(
      &mut file.bump,
      ThmBumpChunk {
        virtual_height: self.virtual_height,
        mode: ThmBumpMode::from(self.bump_mode),
        name: self.bump_name.clone(),
      },
    );
    apply_chunk(&mut file.ext_normal_map_name, self.ext_normal_map_name.clone());
    apply_chunk(&mut file.fade_delay, self.fade_delay);
  }

  /// The descriptor this form describes, built onto `file` when there is one and onto a new one when there is not.
  pub fn to_descriptor(&self, file: Option<ThmFile>) -> ThmFile {
    let mut descriptor: ThmFile = file.unwrap_or_else(ThmFile::new_texture);

    self.apply_to(&mut descriptor);

    descriptor
  }
}

/// Set one chunk without inventing a presence it did not have.
fn apply_chunk<T: Default + PartialEq>(slot: &mut Option<T>, value: T) {
  match slot {
    Some(existing) => *existing = value,
    None if value != T::default() => *slot = Some(value),
    None => {}
  }
}
