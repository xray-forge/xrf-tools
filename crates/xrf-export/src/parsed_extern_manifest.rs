use crate::extern_manifest::ExternManifest;
use crate::parsed_extern::ParsedExtern;

/// A parsed manifest and its non-serialized declaration locations.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedExternManifest {
  pub manifest: ExternManifest,
  pub parsed: Vec<ParsedExtern>,
}
