use serde::Serialize;

/// What one config is to the project holding it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LtxInventoryRole {
  /// Nothing includes it, so it resolves on its own and is a unit a check or a view can be asked for.
  EntryPoint,
  /// Reached only through another config's `#include`; `included_by` names which.
  Included,
  /// A scheme declaration. Not verified against schemes itself, which is why it outranks every other role here.
  SchemeFile,
  /// A file that patches another config rather than standing on its own, as the dialect identified it.
  Attachment,
}
