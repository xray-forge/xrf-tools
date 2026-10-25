use serde::{Deserialize, Serialize};

/// One dialog, as the project index lists it.
///
/// Named for the reduction it is. A descriptor carrying a domain type's own name mirrors that type —
/// [`DialogDescriptor`](crate::DialogDescriptor) mirrors `Dialog` — so a summary of one has to say so.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogSummaryDescriptor {
  pub id: String,
  pub phrases: usize,
  pub priority: Option<i32>,
}
