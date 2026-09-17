use crate::core::types::TauriResult;
use crate::plugins::textures::vocabulary::TextureVocabulary;

/// The names the SDK gives the numbers a descriptor stores.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_vocabulary"))]
#[tauri::command(rename = "get_vocabulary")]
pub async fn textures_get_vocabulary() -> TauriResult<TextureVocabulary> {
  Ok(TextureVocabulary::describe())
}
