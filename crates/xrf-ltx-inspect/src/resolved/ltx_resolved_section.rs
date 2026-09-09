use serde::Serialize;
use xrf_ltx::LtxFieldOrigin;

/// One resolved section with its fields and where each of them came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedSection {
  /// Engine identity of the entry point this section was resolved from, so a consumer can key a cache by it.
  pub entry: String,
  pub name: String,
  /// Parents the header declared, read back from the declaring config.
  pub parents: Vec<String>,
  /// Engine identity of the config whose header declared the section, where the dialect stamped one.
  pub origin: Option<String>,
  /// Fields in resolved order, which is written order with inherited ones folded in ahead of them.
  pub fields: Vec<LtxResolvedField>,
}

/// One resolved field: what it says, and why it says that.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedField {
  pub key: String,
  pub value: String,
  pub origin: LtxResolvedFieldOrigin,
}

/// How one resolved field came to hold the value it holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LtxResolvedFieldOrigin {
  /// Written in the body of the section that holds it.
  Declared { file: Option<String> },
  /// Copied in by inheritance from the section that writes it, which is the ultimate writer and not the parent named
  /// in the header.
  Inherited { section: String, file: Option<String> },
  /// Won a load-order contest under a dialect that ranks statements rather than reading them in order.
  Loaded {
    file: String,
    depth: i32,
    operation: String,
  },
  /// The resolution carries no record for this field, because none was asked for.
  Unrecorded,
}

impl From<Option<&LtxFieldOrigin>> for LtxResolvedFieldOrigin {
  fn from(origin: Option<&LtxFieldOrigin>) -> Self {
    match origin {
      None => Self::Unrecorded,
      Some(LtxFieldOrigin::Declared { file }) => Self::Declared {
        file: file.as_deref().map(String::from),
      },
      Some(LtxFieldOrigin::Inherited { section, file }) => Self::Inherited {
        file: file.as_deref().map(String::from),
        section: String::from(&**section),
      },
      Some(LtxFieldOrigin::Loaded { file, depth, operation }) => Self::Loaded {
        depth: *depth,
        file: String::from(&**file),
        operation: String::from(&**operation),
      },
    }
  }
}
