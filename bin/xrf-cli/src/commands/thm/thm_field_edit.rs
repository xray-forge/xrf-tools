use xrf_thm::{
  ThmBumpChunk, ThmBumpMode, ThmDetailChunk, ThmFile, ThmFormat, ThmMaterial, ThmMaterialChunk, ThmMipFilter,
  ThmTextureFlag, ThmTextureParamChunk, ThmTextureType,
};

/// The descriptor fields one run sets, each left alone where the run names no value for it.
///
/// Each field is set exactly as named. Giving a descriptor a new type does not bring the flags, format and filter the
/// SDK's editor changes with it (`STextureParams::OnTypeChange`); a run that wants those names them too.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ThmFieldEdit {
  pub texture_type: Option<ThmTextureType>,
  pub format: Option<ThmFormat>,
  pub mip_filter: Option<ThmMipFilter>,
  pub bump_mode: Option<ThmBumpMode>,
  pub bump_name: Option<String>,
  pub detail_name: Option<String>,
  pub detail_scale: Option<f32>,
  pub material: Option<ThmMaterial>,
  pub material_weight: Option<f32>,
  pub border_color: Option<u32>,
  pub fade_color: Option<u32>,
  pub fade_amount: Option<u32>,
  pub fade_delay: Option<u8>,
  pub ext_normal_map_name: Option<String>,
  pub virtual_height: Option<f32>,
  pub set_flags: Vec<ThmTextureFlag>,
  pub cleared_flags: Vec<ThmTextureFlag>,
}

impl ThmFieldEdit {
  /// Sets the named fields on `file`, giving it a chunk it lacks, at the SDK's defaults, only where a field in that
  /// chunk is named.
  pub(crate) fn apply_to(&self, file: &mut ThmFile) {
    if let Some(texture_type) = self.texture_type {
      file.texture_type = Some(texture_type);
    }

    if let Some(fade_delay) = self.fade_delay {
      file.fade_delay = Some(fade_delay);
    }

    if let Some(name) = &self.ext_normal_map_name {
      file.ext_normal_map_name = Some(name.clone());
    }

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

  /// Whether the run names a field of the texture parameters.
  fn is_editing_param(&self) -> bool {
    self.format.is_some()
      || self.mip_filter.is_some()
      || self.border_color.is_some()
      || self.fade_color.is_some()
      || self.fade_amount.is_some()
      || !self.set_flags.is_empty()
      || !self.cleared_flags.is_empty()
  }

  fn apply_to_param(&self, param: &mut ThmTextureParamChunk) {
    Self::assign(&mut param.format, self.format);
    Self::assign(&mut param.mip_filter, self.mip_filter);
    Self::assign(&mut param.border_color, self.border_color);
    Self::assign(&mut param.fade_color, self.fade_color);
    Self::assign(&mut param.fade_amount, self.fade_amount);

    for flag in &self.set_flags {
      param.flags.set(*flag, true);
    }

    for flag in &self.cleared_flags {
      param.flags.set(*flag, false);
    }
  }

  fn assign<T>(slot: &mut T, value: Option<T>) {
    if let Some(value) = value {
      *slot = value;
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_thm::{ThmFile, ThmFormat, ThmTextureFlag, ThmTextureParamChunk};

  use super::ThmFieldEdit;

  #[test]
  fn sets_only_the_fields_it_names() {
    let mut file: ThmFile = ThmFile::new_texture();
    let before: ThmTextureParamChunk = file.texture_param.expect("parameters");

    ThmFieldEdit {
      format: Some(ThmFormat::Dxt5),
      detail_name: Some(String::from("detail\\detail_grnd_earth")),
      set_flags: vec![ThmTextureFlag::DiffuseDetail],
      cleared_flags: vec![ThmTextureFlag::DitherColor],
      ..ThmFieldEdit::default()
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
  fn gives_a_descriptor_a_chunk_it_lacks_only_where_a_field_in_it_is_named() {
    let mut file: ThmFile = ThmFile::default();

    ThmFieldEdit {
      bump_name: Some(String::from("wpn\\wpn_pm_bump")),
      ..ThmFieldEdit::default()
    }
    .apply_to(&mut file);

    assert_eq!(file.bump.map(|bump| bump.name), Some(String::from("wpn\\wpn_pm_bump")));
    assert_eq!(file.texture_param, None);
    assert_eq!(file.detail, None);
  }
}
