use serde::Serialize;

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
      sections: Vec::new(),
    }
  }
}

/// One section header, and what resolving the file it belongs to made of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureSection {
  /// One-based line the header was written on.
  pub line: u32,
  /// The name exactly as bracketed, padding included - `[ wpn_base ]` is not `[wpn_base]` and nothing inherits it.
  pub name: String,
  /// The header's operation prefix, spelled as the engine spells it: empty, `!`, `@` or `!!`.
  pub operation: String,
  pub parents: Vec<LtxStructureParent>,
  /// The scheme the resolved section is bound to, directly or through a parent.
  pub scheme: Option<LtxStructureScheme>,
}

/// One name after the `:` of a header, and whether the resolution holds it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureParent {
  pub name: String,
  /// Whether a section by this name exists in the resolution this file was read against.
  pub resolves: bool,
}

/// The `$scheme` a resolved section ends up carrying, and whether the project declares it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureScheme {
  pub name: String,
  /// Whether a scheme file declares it. False is itself a finding, and the verifier reports it as one.
  pub is_declared: bool,
}

/// One `#include`, and the configs it actually reached.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureInclude {
  /// One-based line the statement was written on.
  pub line: u32,
  /// The path or wildcard mask as authored.
  pub statement: String,
  /// Engine identities the statement resolved to. Empty means it reached nothing.
  pub resolved: Vec<String>,
}

/// Why one config would not parse.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureParseError {
  /// One-based line the parser stopped on.
  pub line: u32,
  /// One-based column the parser stopped on.
  pub column: u32,
  pub message: String,
}
