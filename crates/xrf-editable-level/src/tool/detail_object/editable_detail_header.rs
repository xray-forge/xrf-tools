/// The detail grid's header, `DetailHeader` (`Layers/xrRender/DetailFormat.h`), as the tool stores it: 24 bytes cast
/// onto the struct.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditableDetailHeader {
  /// `DETAIL_VERSION` once the tool has exported, else what the tool last held.
  pub version: u32,
  pub object_count: u32,
  /// The slot the level's origin falls in, and the grid's size in slots.
  pub offset_x: i32,
  pub offset_z: i32,
  pub size_x: u32,
  pub size_z: u32,
}
