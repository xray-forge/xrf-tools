use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// One place of a listed slot: culled on its own, then drawn as its slot's clusters.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Row")]
pub struct StaticRow {
  /// The place's sphere in renderer space; a negative radius is none.
  pub sphere: Vec4,
  pub place: u32,
  pub slot: u32,
  /// The impostor whose level of detail decides it, [`StaticRow::NO_LOD`] for none.
  pub lod: u32,
  /// Its progressive band, the bands its mesh has and the windows they share: `band | bands << 4 | windows << 8`.
  pub band: u32,
}

impl StaticRow {
  pub const NO_LOD: u32 = u32::MAX;

  /// The most slide windows a band word counts, its sixteen bits' worth; a mesh with more is graded as if it had
  /// these.
  pub const MAX_WINDOWS: u32 = 0xFFFF;

  /// A progressive band, of the bands its mesh has, in four bits each, and the windows they share in sixteen.
  pub const fn pack_band(band: u32, bands: u32, windows: u32) -> u32 {
    let windows: u32 = if windows > Self::MAX_WINDOWS {
      Self::MAX_WINDOWS
    } else {
      windows
    };

    (band & 0xF) | ((bands & 0xF) << 4) | (windows << 8)
  }

  /// Its progressive band, nought the whole detail.
  pub const fn get_band(&self) -> u32 {
    self.band & 0xF
  }
}
