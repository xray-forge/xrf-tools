use glam::Vec4;

/// One place of a listed slot: culled on its own, then drawn as its slot's clusters.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticRow {
  /// The place's sphere in renderer space; a negative radius is none.
  pub sphere: Vec4,
  pub place: u32,
  pub slot: u32,
  /// The impostor whose level of detail decides it, [`StaticRow::NO_LOD`] for none; the top bit marks the impostor's
  /// own row.
  pub lod: u32,
  /// Its progressive band, the bands its mesh has and the windows they share: `band | bands << 8 | windows << 16`.
  pub band: u32,
}

impl StaticRow {
  pub const NO_LOD: u32 = u32::MAX;

  pub const fn pack_band(band: u32, bands: u32, windows: u32) -> u32 {
    band | (bands << 8) | (windows << 16)
  }
}
