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
  /// Its progressive band, the bands its mesh has, the windows they share and the visibility group showing it:
  /// `band | bands << 4 | windows << 8 | group << 24`, a group of zero always shown.
  pub band: u32,
}

impl StaticRow {
  pub const NO_LOD: u32 = u32::MAX;

  /// The most slide windows a band word counts, its sixteen bits' worth; a mesh with more is graded as if it had
  /// these.
  pub const MAX_WINDOWS: u32 = 0xFFFF;

  /// A progressive band, of the bands its mesh has, in four bits each, and the windows they share in sixteen, which
  /// leaves the top byte to [`StaticRow::pack_group`]: a mesh of hundreds of windows must not spill into it.
  pub const fn pack_band(band: u32, bands: u32, windows: u32) -> u32 {
    let windows: u32 = if windows > Self::MAX_WINDOWS {
      Self::MAX_WINDOWS
    } else {
      windows
    };

    (band & 0xF) | ((bands & 0xF) << 4) | (windows << 8)
  }

  /// A visibility group's bits, from one; at most eight groups.
  pub const fn pack_group(group: u32) -> u32 {
    group << 24
  }
}
