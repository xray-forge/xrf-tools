use serde::{Deserialize, Serialize};

/// A share of what the GPU allocator holds allocated: the resources made under one name, their bytes and how many.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMemorySection {
  /// The name the resources were made under, their trailing number dropped; what the largest leave over is "Other".
  pub name: String,
  pub bytes: u64,
  pub count: u32,
}
