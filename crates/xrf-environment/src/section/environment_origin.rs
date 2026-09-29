use serde::Serialize;
use xrf_ltx::LtxFieldOrigin;

/// Where a key's resolved value is written, for a writer that has to put an edit back where it came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EnvironmentOrigin {
  /// Written in the section itself.
  Declared { file: Option<String> },
  /// Inherited from the section that writes it.
  Inherited { section: String, file: Option<String> },
  /// Won a load-order contest under a patching dialect, DLTX's `mod_` files among them.
  Loaded {
    file: String,
    depth: i32,
    operation: String,
  },
}

impl From<&LtxFieldOrigin> for EnvironmentOrigin {
  fn from(origin: &LtxFieldOrigin) -> Self {
    match origin {
      LtxFieldOrigin::Declared { file } => Self::Declared {
        file: file.as_deref().map(str::to_owned),
      },
      LtxFieldOrigin::Inherited { section, file } => Self::Inherited {
        file: file.as_deref().map(str::to_owned),
        section: section.to_string(),
      },
      LtxFieldOrigin::Loaded { file, depth, operation } => Self::Loaded {
        depth: *depth,
        file: file.to_string(),
        operation: operation.to_string(),
      },
    }
  }
}
