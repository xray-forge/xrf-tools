use crate::core::types::TauriResult;
use crate::plugins::textures::vocabulary::TextureVocabulary;

/// The names the SDK gives the numbers a descriptor stores.
///
/// A static table, asked for once when the editor opens. It takes no roots and reads nothing off disk: what a `tfDXT5`
/// or a `flBinaryAlpha` is called is a property of the format, not of the tree being edited.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_vocabulary"))]
#[tauri::command(rename = "get_vocabulary")]
pub async fn textures_get_vocabulary() -> TauriResult<TextureVocabulary> {
  Ok(TextureVocabulary::describe())
}
