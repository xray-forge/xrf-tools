use serde::Serialize;

use crate::structure::{LtxStructureEntry, LtxStructureInclude, LtxStructureParseError, LtxStructureSection};

/// One config as written, carrying only what reading the text cannot answer.
///
/// Keys and values are intentionally absent: a highlighter already colours them from the line itself, and a config tree
/// holds hundreds of thousands of them. What travels is what needs the parser's truth - where a section begins, whether
/// its parents resolve, which scheme it ends up bound to, and where an include actually landed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxFileStructure {
  /// Engine identity of the config this describes.
  pub path: String,
  /// Entry points whose resolution reaches this config, in project order.
  ///
  /// Empty for a config nothing includes and that is not itself an entry point, which under a patch dialect means an
  /// attachment.
  pub entry_points: Vec<String>,
  pub sections: Vec<LtxStructureSection>,
  /// Keys written before the first section header, in file order.
  pub root_entries: Vec<LtxStructureEntry>,
  pub includes: Vec<LtxStructureInclude>,
  /// Why the file did not parse, when it did not.
  ///
  /// Present means `sections` and `includes` are empty because nothing could be read, not because the file holds
  /// neither.
  pub parse_error: Option<LtxStructureParseError>,
}

impl LtxFileStructure {
  /// A config no entry point reaches, and therefore no resolution judges.
  ///
  /// Under a patch dialect that is an attachment: its sections do reach a resolution, folded into the config it
  /// patches, but nothing records which config that is - `LtxDialect::plan_attachments` answers a flat list of names
  /// and drops the pairing. So the honest answer is that this file was not judged, rather than judging it against a
  /// resolution it does not belong to and reporting every parent of every section as unresolved.
  ///
  /// A viewer still shows the text; only the layer that needs the parser's truth is absent.
  pub fn new_unreached(path: &str) -> Self {
    Self {
      entry_points: Vec::new(),
      includes: Vec::new(),
      parse_error: None,
      path: String::from(path),
      root_entries: Vec::new(),
      sections: Vec::new(),
    }
  }
}
