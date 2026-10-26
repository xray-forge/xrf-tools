use serde::Serialize;
use xrf_ogf::OgfDescriptionChunk;

/// Who built a visual and with what, as its description chunk records it; the times are the Unix seconds the file
/// stores.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfDescriptionReport {
  built_at: u64,
  convertor: String,
  created_at: u64,
  creator: String,
  edited_at: u64,
  editor: String,
  source_file: String,
}

impl OgfDescriptionReport {
  pub fn new(description: &OgfDescriptionChunk) -> Self {
    Self {
      built_at: description.built_at,
      convertor: description.convertor.clone(),
      created_at: description.created_at,
      creator: description.creator.clone(),
      edited_at: description.edited_at,
      editor: description.editor.clone(),
      source_file: description.source_file.clone(),
    }
  }
}
