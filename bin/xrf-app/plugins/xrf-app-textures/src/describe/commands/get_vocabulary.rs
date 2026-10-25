use xrf_app_core::TauriResult;

use crate::describe::TextureVocabulary;

/// The names the SDK gives the numbers a descriptor stores.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "get_vocabulary")]
pub async fn textures_get_vocabulary() -> TauriResult<TextureVocabulary> {
  Ok(TextureVocabulary::describe())
}
