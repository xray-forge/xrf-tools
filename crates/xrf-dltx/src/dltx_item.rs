use xrf_ltx::LtxKeyOperation;

/// One field line as the engine records it, with everything needed to rank it against another.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltxItem {
  /// Field name with its case as authored. The engine lowercases section names and never key names.
  pub key: String,
  /// Value as authored, or `None` for a bare key. A deletion carries no value: the engine discards whatever was
  /// written after `!key` (`Xr_ini.cpp`).
  pub value: Option<String>,
  pub operation: LtxKeyOperation,
  /// Lowercased base name of the file this came from, which is the provenance the engine keeps.
  pub filename: String,
  /// Load rank. Base root is 0, each include level one more, each mod file 200 less. Lower wins.
  pub depth: i32,
  /// Position within its file, breaking ties at equal depth. Higher wins for a field, ascending order for a list
  /// operation (`Xr_ini.cpp`).
  pub insertion_index: u32,
}

impl DltxItem {
  /// The value a resolved field would take, treating a bare key as empty.
  pub fn to_resolved_value(&self) -> String {
    self.value.clone().unwrap_or_default()
  }
}
