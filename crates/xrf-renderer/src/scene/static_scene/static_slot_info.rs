/// What one slot is a part of, kept on the CPU to turn a pick back into what the page names.
#[derive(Clone, Copy, Debug)]
pub struct StaticSlotInfo {
  pub sector: u32,
  pub shader_id: u16,
  /// The sector's instanced mesh, by its index in the pack, or none for its baked geometry.
  pub mesh: Option<u32>,
  /// The mesh's first place, which a picked place counts its instance from.
  pub first_place: u32,
}

impl StaticSlotInfo {
  /// What a slot of a spawned model names for its sector: none.
  pub const NO_SECTOR: u32 = u32::MAX;
}
