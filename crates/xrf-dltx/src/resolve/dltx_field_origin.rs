use std::sync::Arc;

use xrf_ltx::LtxKeyOperation;

use crate::load::dltx_item::DltxItem;

/// Where one resolved field came from.
///
/// The engine keeps this per item and exposes it to Lua, because in a patched install the only way to explain a value
/// is to name the file that won it (`xr_ini.h`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltxFieldOrigin {
  /// Lowercased base name of the winning file, as the handle the loader interned.
  pub file: Arc<str>,
  /// Load rank of the winning statement. Negative means a mod file.
  pub depth: i32,
  /// Which operation produced the value.
  pub operation: LtxKeyOperation,
}

impl DltxFieldOrigin {
  pub fn of(item: &DltxItem) -> Self {
    Self {
      depth: item.depth,
      file: Arc::clone(&item.filename),
      operation: item.operation,
    }
  }

  /// Whether a mod file rather than the base tree set this value.
  ///
  /// The engine answers the same question by testing the file name for a `mod_` prefix.
  pub fn is_from_mod_file(&self) -> bool {
    self.file.starts_with("mod_")
  }
}
