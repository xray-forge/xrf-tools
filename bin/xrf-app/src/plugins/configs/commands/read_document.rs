use std::sync::Arc;

use tauri::State;
use xrf_ltx::{LtxDocumentSource, LtxResolution};
use xrf_ltx_inspect::{LtxFileStructure, LtxFileText, LtxRootReader, LtxTextReader};
use xrf_utils::encode_w1251_bytes_to_string;
use xrf_vfs::XrayLogicalPath;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsDocument;
use crate::plugins::configs::request::ConfigsReadDocumentRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// Reads one config as the authored view renders it: its lines, and what only the parser knows about them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_document"))]
#[tauri::command(rename = "read_document")]
pub async fn configs_read_document(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsReadDocumentRequest,
) -> TauriResult<ConfigsDocument> {
  let ConfigsReadDocumentRequest { session_id, path } = request;

  // Taken before the hop and the guard dropped with it, so the work does not hold the session against every other
  // command. A snapshot outlives its state: closing the project mid-read leaves this answering what it was asked for,
  // which is sound because reading commits nothing back.
  let opened: Arc<ConfigsProject> = state.require(session_id)?;

  // Bounded by the include tree of whichever entry point the file belongs to, which on a game tree is `system.ltx` and
  // most of the configs under it.
  execution
    .run_blocking("Configs document read", move || read_document(&opened, &path))
    .await
    .and_then(|document| document)
}

/// The read itself: the text as authored, and the structure judged against its entry point's resolution.
fn read_document(opened: &ConfigsProject, path: &str) -> TauriResult<ConfigsDocument> {
  let logical: XrayLogicalPath = XrayLogicalPath::new(path).map_err(|error| error.to_string())?;

  let bytes: Vec<u8> = opened
    .project
    .vfs()
    .scoped(opened.project.scope())
    .read_bytes(logical.as_str())
    .map_err(|error| format!("Cannot read '{path}': {error}"))?;

  // Configs are strict Windows-1251, loose and archived alike; a lossy decode would put a viewer and a writer at odds
  // about what the file says.
  let contents: String =
    encode_w1251_bytes_to_string(&bytes).map_err(|error| format!("Cannot decode '{path}': {error}"))?;

  let text: LtxFileText = LtxTextReader::read(logical.as_str(), &contents);

  // An entry point answers itself, so this is the file's own resolution when nothing includes it. A config reached by
  // two entry points takes the first in project order; the frontend is told which through the structure it gets back.
  let entry_points: Vec<String> = opened.descriptor.inventory.list_entry_points_of(logical.as_str());

  let Some(first) = entry_points.first() else {
    // A config the project holds but no entry point reaches: an attachment under a dialect that has them. Its text is
    // still worth showing, and there is no resolution it participates in to judge it against.
    return Ok(ConfigsDocument {
      structure: LtxFileStructure::new_unreached(logical.as_str()),
      text,
    });
  };

  let entry: XrayLogicalPath = XrayLogicalPath::new(first).map_err(|error| error.to_string())?;
  let resolution: Arc<LtxResolution> = opened.resolve(&entry)?;
  let source = opened.project.document_source();
  let declared: Vec<&str> = opened
    .descriptor
    .declared_schemes
    .iter()
    .map(String::as_str)
    .collect::<Vec<&str>>();

  let structure: LtxFileStructure = LtxRootReader::new(
    entry.as_str(),
    opened.project.get_dialect().get_name(),
    &resolution,
    &source as &dyn LtxDocumentSource,
  )
  .with_declared_schemes(&declared)
  .read_structure(logical.as_str(), &entry_points)
  .map_err(|error| format!("Cannot read the structure of '{path}': {error}"))?;

  Ok(ConfigsDocument { structure, text })
}
