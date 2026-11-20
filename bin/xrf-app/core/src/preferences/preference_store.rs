use serde_json::Value;
use tauri::Runtime;
use tauri_plugin_store::Store;

/// The store preferences are kept in, whatever runtime opened it, so [`crate::preferences::Preferences`] is one type
/// a plugin can hold as managed state.
pub(crate) trait PreferenceStore: Send + Sync {
  fn get(&self, key: &str) -> Option<Value>;

  fn set(&self, key: &str, value: Value);

  fn save(&self) -> tauri_plugin_store::Result<()>;
}

impl<R: Runtime> PreferenceStore for Store<R> {
  fn get(&self, key: &str) -> Option<Value> {
    Self::get(self, key)
  }

  fn set(&self, key: &str, value: Value) {
    Self::set(self, key, value);
  }

  fn save(&self) -> tauri_plugin_store::Result<()> {
    Self::save(self)
  }
}
